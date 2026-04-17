const fs = require('fs');
const path = require('path');
const builtinModules = require('module').builtinModules;

function getFiles(dir, filesList = []) {
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const filePath = path.join(dir, file);
    if (fs.statSync(filePath).isDirectory()) {
      getFiles(filePath, filesList);
    } else if (filePath.endsWith('.ts') || filePath.endsWith('.js') || filePath.endsWith('.tsx') || filePath.endsWith('.jsx')) {
      filesList.push(filePath);
    }
  }
  return filesList;
}

const srcDir = path.join(__dirname, 'restored-src', 'src');
const allFiles = getFiles(srcDir);

const importRegex = /import\s+(?:[\s\w,{}*]+)\s+from\s+['"]([^'"]+)['"]/g;
const importSideEffectRegex = /import\s+['"]([^'"]+)['"]/g;
const requireRegex = /require\(['"]([^'"]+)['"]\)/g;
const dynamicImportRegex = /import\(['"]([^'"]+)['"]\)/g;

let missingCount = 0;
const missingModules = new Set();
const missingRelative = new Set();

const nodeBuiltins = new Set(builtinModules);
for (const m of builtinModules) {
  nodeBuiltins.add(`node:${m}`);
}

function checkImport(imp, file) {
  if (nodeBuiltins.has(imp)) return; // Built-in
  if (imp.startsWith('bun:')) return; // Bun built-in

  let targetPath;
  let isRelative = false;

  if (imp.startsWith('.') || imp.startsWith('/')) {
    isRelative = true;
    targetPath = path.resolve(path.dirname(file), imp);
  } else if (imp.startsWith('src/')) {
    isRelative = true;
    targetPath = path.resolve(__dirname, 'restored-src', imp);
  }

  if (isRelative) {
    // If it ends with .js, we should also check if a .ts or .tsx file exists without the .js
    let baseTargetPath = targetPath;
    if (targetPath.endsWith('.js')) {
      baseTargetPath = targetPath.slice(0, -3);
    } else if (targetPath.endsWith('.jsx')) {
      baseTargetPath = targetPath.slice(0, -4);
    }

    const exts = ['', '.js', '.ts', '.jsx', '.tsx', '/index.js', '/index.ts', '/index.jsx', '/index.tsx'];
    let found = false;
    
    // First check exact targetPath
    for (const ext of exts) {
      if (fs.existsSync(baseTargetPath + ext)) {
        found = true;
        break;
      }
    }
    
    // Fallback: check original targetPath + extensions (just in case)
    if (!found) {
      for (const ext of exts) {
        if (fs.existsSync(targetPath + ext)) {
          found = true;
          break;
        }
      }
    }

    if (!found) {
      missingRelative.add(`${imp} (in ${path.relative(__dirname, file)})`);
      missingCount++;
    }
  } else {
    // Bare module
    const parts = imp.split('/');
    const moduleName = imp.startsWith('@') ? `${parts[0]}/${parts[1]}` : parts[0];
    
    const nodeModulesPath = path.join(__dirname, 'restored-src', 'node_modules', moduleName);
    const vendorPath = path.join(__dirname, 'restored-src', 'vendor', moduleName);
    
    if (!fs.existsSync(nodeModulesPath) && !fs.existsSync(vendorPath)) {
      missingModules.add(`${imp} (in ${path.relative(__dirname, file)})`);
      missingCount++;
    }
  }
}

for (const file of allFiles) {
  const content = fs.readFileSync(file, 'utf8');
  let match;
  
  while ((match = importRegex.exec(content)) !== null) checkImport(match[1], file);
  while ((match = importSideEffectRegex.exec(content)) !== null) checkImport(match[1], file);
  while ((match = requireRegex.exec(content)) !== null) checkImport(match[1], file);
  while ((match = dynamicImportRegex.exec(content)) !== null) checkImport(match[1], file);
}

console.log(`Checked ${allFiles.length} files.`);
console.log(`Found ${missingCount} missing imports/dependencies.`);

if (missingModules.size > 0) {
  console.log('\nMissing Modules/Packages:');
  for (const m of Array.from(missingModules).slice(0, 50)) console.log(` - ${m}`);
  if (missingModules.size > 50) console.log(`   ...and ${missingModules.size - 50} more`);
}

if (missingRelative.size > 0) {
  console.log('\nMissing Relative Imports:');
  for (const m of Array.from(missingRelative).slice(0, 50)) console.log(` - ${m}`);
  if (missingRelative.size > 50) console.log(`   ...and ${missingRelative.size - 50} more`);
}
