use anyhow::{Context, Result};
use enigo::{Enigo, Keyboard, Mouse, Coordinate, Button, Direction};
use std::time::Duration;
use tokio::time::sleep;

const MOVE_SETTLE_MS: u64 = 50;

pub struct ComputerExecutor {
    enigo: std::sync::Mutex<Enigo>,
}

impl ComputerExecutor {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&enigo::Settings::default())
            .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {:?}", e))?;
            
        Ok(Self {
            enigo: std::sync::Mutex::new(enigo),
        })
    }

    /// Move mouse and wait for system events to settle
    pub async fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        {
            let mut enigo = self.enigo.lock().unwrap();
            enigo.move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| anyhow::anyhow!("Failed to move mouse: {:?}", e))?;
        }
        sleep(Duration::from_millis(MOVE_SETTLE_MS)).await;
        Ok(())
    }

    /// Click a mouse button a specified number of times
    pub async fn click(&self, x: i32, y: i32, button: Button, count: u32) -> Result<()> {
        self.move_mouse(x, y).await?;
        
        let mut enigo = self.enigo.lock().unwrap();
        for _ in 0..count {
            enigo.button(button, Direction::Click)
                .map_err(|e| anyhow::anyhow!("Failed to click: {:?}", e))?;
        }
        
        Ok(())
    }

    /// Perform an animated drag motion
    pub async fn drag(&self, start: Option<(i32, i32)>, end_x: i32, end_y: i32, animated: bool) -> Result<()> {
        if let Some((x, y)) = start {
            self.move_mouse(x, y).await?;
        }
        
        {
            let mut enigo = self.enigo.lock().unwrap();
            enigo.button(Button::Left, Direction::Press)
                .map_err(|e| anyhow::anyhow!("Failed to press mouse: {:?}", e))?;
        }
        
        sleep(Duration::from_millis(MOVE_SETTLE_MS)).await;
        
        let result = if animated {
            self.animated_move(end_x, end_y).await
        } else {
            self.move_mouse(end_x, end_y).await
        };
        
        // Always ensure we release the mouse, even if the move failed
        {
            let mut enigo = self.enigo.lock().unwrap();
            let _ = enigo.button(Button::Left, Direction::Release);
        }
        
        result
    }

    /// Type text character by character
    pub async fn type_text(&self, text: &str) -> Result<()> {
        let mut enigo = self.enigo.lock().unwrap();
        enigo.text(text)
            .map_err(|e| anyhow::anyhow!("Failed to type text: {:?}", e))?;
        Ok(())
    }

    /// Execute a key sequence like 'ctrl+shift+a'
    pub async fn key_sequence(&self, keys: &[enigo::Key]) -> Result<()> {
        let mut enigo = self.enigo.lock().unwrap();
        
        // Press all keys
        for key in keys {
            enigo.key(*key, Direction::Press)
                .map_err(|e| anyhow::anyhow!("Failed to press key: {:?}", e))?;
        }
        
        // Release in reverse order
        for key in keys.iter().rev() {
            let _ = enigo.key(*key, Direction::Release);
        }
        
        Ok(())
    }

    /// Scroll the mouse wheel
    pub async fn scroll(&self, x: i32, y: i32, dx: i32, dy: i32) -> Result<()> {
        self.move_mouse(x, y).await?;
        
        let mut enigo = self.enigo.lock().unwrap();
        if dy != 0 {
            enigo.scroll(dy, enigo::Axis::Vertical)
                .map_err(|e| anyhow::anyhow!("Failed to scroll vertical: {:?}", e))?;
        }
        if dx != 0 {
            enigo.scroll(dx, enigo::Axis::Horizontal)
                .map_err(|e| anyhow::anyhow!("Failed to scroll horizontal: {:?}", e))?;
        }
        Ok(())
    }
    
    async fn animated_move(&self, target_x: i32, target_y: i32) -> Result<()> {
        let (start_x, start_y) = {
            let enigo = self.enigo.lock().unwrap();
            enigo.main_display().map_err(|e| anyhow::anyhow!("Failed to get cursor pos: {:?}", e))?
        };
        
        let delta_x = target_x - start_x;
        let delta_y = target_y - start_y;
        let distance = ((delta_x.pow(2) + delta_y.pow(2)) as f64).sqrt();
        
        if distance < 1.0 {
            return Ok(());
        }
        
        let duration_sec = (distance / 2000.0).min(0.5);
        if duration_sec < 0.03 {
            return self.move_mouse(target_x, target_y).await;
        }
        
        let frame_rate = 60.0;
        let total_frames = (duration_sec * frame_rate) as i32;
        let frame_interval_ms = (1000.0 / frame_rate) as u64;
        
        for frame in 1..=total_frames {
            let t = frame as f64 / total_frames as f64;
            let eased = 1.0 - (1.0 - t).powi(3); // Ease-out-cubic
            
            let x = start_x + (delta_x as f64 * eased).round() as i32;
            let y = start_y + (delta_y as f64 * eased).round() as i32;
            
            {
                let mut enigo = self.enigo.lock().unwrap();
                let _ = enigo.move_mouse(x, y, Coordinate::Abs);
            }
            
            if frame < total_frames {
                sleep(Duration::from_millis(frame_interval_ms)).await;
            }
        }
        
        sleep(Duration::from_millis(MOVE_SETTLE_MS)).await;
        Ok(())
    }
    
    pub async fn read_clipboard(&self) -> Result<String> {
        let output = tokio::process::Command::new("pbpaste")
            .output()
            .await
            .context("Failed to execute pbpaste")?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("pbpaste failed"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
    
    pub async fn write_clipboard(&self, text: &str) -> Result<()> {
        let mut child = tokio::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn pbcopy")?;
            
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(text.as_bytes()).await?;
        }
        
        let status = child.wait().await?;
        if !status.success() {
            return Err(anyhow::anyhow!("pbcopy failed"));
        }
        
        Ok(())
    }
}
