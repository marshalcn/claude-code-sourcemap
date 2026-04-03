use anyhow::Result;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaStatus {
    Allowed,
    AllowedWarning,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitType {
    FiveHour,
    SevenDay,
    SevenDayOpus,
    SevenDaySonnet,
    Overage,
    Unknown(String),
}

impl From<&str> for RateLimitType {
    fn from(s: &str) -> Self {
        match s {
            "five_hour" => RateLimitType::FiveHour,
            "seven_day" => RateLimitType::SevenDay,
            "seven_day_opus" => RateLimitType::SevenDayOpus,
            "seven_day_sonnet" => RateLimitType::SevenDaySonnet,
            "overage" => RateLimitType::Overage,
            other => RateLimitType::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverageDisabledReason {
    OverageNotProvisioned,
    OrgLevelDisabled,
    OutOfCredits,
    MemberLevelDisabled,
    NoLimitsConfigured,
    Unknown(String),
}

impl From<&str> for OverageDisabledReason {
    fn from(s: &str) -> Self {
        match s {
            "overage_not_provisioned" => OverageDisabledReason::OverageNotProvisioned,
            "org_level_disabled" => OverageDisabledReason::OrgLevelDisabled,
            "out_of_credits" => OverageDisabledReason::OutOfCredits,
            "member_level_disabled" => OverageDisabledReason::MemberLevelDisabled,
            "no_limits_configured" => OverageDisabledReason::NoLimitsConfigured,
            other => OverageDisabledReason::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeAiLimits {
    pub status: QuotaStatus,
    pub unified_rate_limit_fallback_available: bool,
    pub resets_at: Option<u64>,
    pub rate_limit_type: Option<RateLimitType>,
    pub utilization: Option<f64>,
    pub overage_status: Option<QuotaStatus>,
    pub overage_resets_at: Option<u64>,
    pub overage_disabled_reason: Option<OverageDisabledReason>,
    pub is_using_overage: bool,
    pub surpassed_threshold: Option<f64>,
}

impl Default for ClaudeAiLimits {
    fn default() -> Self {
        Self {
            status: QuotaStatus::Allowed,
            unified_rate_limit_fallback_available: false,
            resets_at: None,
            rate_limit_type: None,
            utilization: None,
            overage_status: None,
            overage_resets_at: None,
            overage_disabled_reason: None,
            is_using_overage: false,
            surpassed_threshold: None,
        }
    }
}

pub struct RateLimitTracker {
    pub current_limits: Arc<Mutex<ClaudeAiLimits>>,
}

impl RateLimitTracker {
    pub fn new() -> Self {
        Self {
            current_limits: Arc::new(Mutex::new(ClaudeAiLimits::default())),
        }
    }

    /// Parse headers from Anthropic API to update the current limits
    pub async fn extract_quota_status_from_headers(&self, headers: &HeaderMap) -> Result<()> {
        let mut limits = ClaudeAiLimits::default();

        if let Some(status_val) = headers.get("anthropic-ratelimit-unified-status") {
            let s = status_val.to_str().unwrap_or("allowed");
            limits.status = match s {
                "rejected" => QuotaStatus::Rejected,
                "allowed_warning" => QuotaStatus::AllowedWarning,
                _ => QuotaStatus::Allowed,
            };
        }

        if let Some(reset_val) = headers.get("anthropic-ratelimit-unified-reset") {
            if let Ok(s) = reset_val.to_str() {
                limits.resets_at = s.parse::<u64>().ok();
            }
        }

        if let Some(fallback) = headers.get("anthropic-ratelimit-unified-fallback") {
            limits.unified_rate_limit_fallback_available = fallback.to_str().unwrap_or("") == "available";
        }

        if let Some(rl_type) = headers.get("anthropic-ratelimit-unified-representative-claim") {
            limits.rate_limit_type = Some(RateLimitType::from(rl_type.to_str().unwrap_or("")));
        }

        if let Some(overage_status) = headers.get("anthropic-ratelimit-unified-overage-status") {
            let s = overage_status.to_str().unwrap_or("allowed");
            limits.overage_status = Some(match s {
                "rejected" => QuotaStatus::Rejected,
                "allowed_warning" => QuotaStatus::AllowedWarning,
                _ => QuotaStatus::Allowed,
            });
        }

        if let Some(reason) = headers.get("anthropic-ratelimit-unified-overage-disabled-reason") {
            limits.overage_disabled_reason = Some(OverageDisabledReason::from(reason.to_str().unwrap_or("")));
        }

        // Determine if we're using overage (standard limits rejected but overage allowed)
        limits.is_using_overage = limits.status == QuotaStatus::Rejected && 
            (limits.overage_status == Some(QuotaStatus::Allowed) || limits.overage_status == Some(QuotaStatus::AllowedWarning));

        // Early warning checks
        let claims = vec![("five_hour", "5h"), ("seven_day", "7d"), ("overage", "overage")];
        
        for (claim_type, abbrev) in claims {
            let thresh_key = format!("anthropic-ratelimit-unified-{}-surpassed-threshold", abbrev);
            if let Some(thresh_val) = headers.get(&thresh_key) {
                if let Ok(s) = thresh_val.to_str() {
                    if let Ok(threshold) = s.parse::<f64>() {
                        limits.status = QuotaStatus::AllowedWarning;
                        limits.surpassed_threshold = Some(threshold);
                        limits.rate_limit_type = Some(RateLimitType::from(claim_type));
                        
                        let util_key = format!("anthropic-ratelimit-unified-{}-utilization", abbrev);
                        if let Some(util_val) = headers.get(&util_key) {
                            if let Ok(us) = util_val.to_str() {
                                limits.utilization = us.parse::<f64>().ok();
                            }
                        }
                        break;
                    }
                }
            }
        }

        let mut current = self.current_limits.lock().await;
        if *current != limits {
            // In a real application, emit an event here to update the UI
            *current = limits;
        }

        Ok(())
    }
}
