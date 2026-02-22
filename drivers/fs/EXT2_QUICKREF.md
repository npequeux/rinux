# ext2 Filesystem - Quick Reference

## Quick Start

### Mounting an ext2 Filesystem

```rust
use rinux_fs::ext2::Ext2Filesystem;
use alloc::sync::Arc;

// Get block device from AHCI driver
let device = rinux_block::get_device(0)?;

// Mount ext2 filesystem
let fs = Ext2Filesystem::mount(device)?;

// Set as root filesystem
rinux_fs::mount::set_root(fs.clone())?;
```

### Reading a File

```rust
// Get root directory
let root = fs.root();

// Lookup file
let file = root.lookup("myfile.txt")?;

// Get file size
let attr = file.getattr()?;

// Read contents
let mut buffer = vec![0u8; attr.size as usize];
file.read(0, &mut buffer)?;
```

### Writing a File

```rust
use rinux_fs::vfs::FileMode;

// Create file
let mode = FileMode::new(0o644); // rw-r--r--
let file = root.create("newfile.txt", mode)?;

// Write data
let data = b"Hello, ext2!";
file.write(0, data)?;

// Sync to disk
file.fsync()?;
```

### Directory Operations

```rust
// Create directory
let mode = FileMode::new(0o755); // rwxr-xr-x
let dir = root.mkdir("mydir", mode)?;

// List contents
let entries = root.readdir()?;
for entry in entries {
    println!("{}: inode {}", entry.name, entry.ino);
}

// Remove directory
root.rmdir("mydir")?;
```

### Symbolic Links

```rust
// Create symlink
let link = root.symlink("readme", "README.txt")?;

// Read target
let target = link.readlink()?;
```

## API Reference

### Ext2Filesystem

| Method | Description |
|--------|-------------|
| `mount(device)` | Mount ext2 from block device |
| `root()` | Get root VNode |
| `sync()` | Flush all cached data |
| `statfs()` | Get filesystem statistics |
| `unmount()` | Unmount filesystem |

### VNode Operations

| Method | Description |
|--------|-------------|
| `read(offset, buffer)` | Read file data |
| `write(offset, buffer)` | Write file data |
| `getattr()` | Get file attributes |
| `setattr(attr)` | Set file attributes |
| `readdir()` | List directory contents |
| `lookup(name)` | Find entry by name |
| `create(name, mode)` | Create new file |
| `mkdir(name, mode)` | Create new directory |
| `unlink(name)` | Remove file |
| `rmdir(name)` | Remove directory |
| `symlink(name, target)` | Create symbolic link |
| `readlink()` | Read link target |
| `truncate(size)` | Resize file |
| `fsync()` | Sync file to disk |

## File Permissions

| Mode | Octal | Description |
|------|-------|-------------|
| Read all | 0o444 | r--r--r-- |
| Read/Write owner, read others | 0o644 | rw-r--r-- |
| Full owner, read others | 0o744 | rwxr--r-- |
| Full owner, read/execute others | 0o755 | rwxr-xr-x |
| Full all | 0o777 | rwxrwxrwx |

## Error Handling

All operations return `Result<T, FsError>`:

| Error | Description |
|-------|-------------|
| `NotFound` | File/directory not found |
| `PermissionDenied` | Access denied |
| `AlreadyExists` | File already exists |
| `NotADirectory` | Operation requires directory |
| `IsADirectory` | Operation invalid on directory |
| `NotEmpty` | Directory not empty |
| `InvalidArgument` | Invalid parameter |
| `NoSpaceLeft` | Disk full |
| `IoError` | Device I/O error |

## Performance Tips

1. **Use block cache**: Default 256 blocks is good for most uses
2. **Batch operations**: Group file operations together
3. **Sync strategically**: Don't sync after every write
4. **Read whole files**: More efficient than many small reads
5. **Use sparse files**: Leave holes for zero regions

## Block Sizes

| Block Size | Max File Size | Blocks per Group |
|------------|---------------|------------------|
| 1024 bytes | ~16 GB | 8,192 |
| 2048 bytes | ~256 GB | 16,384 |
| 4096 bytes | ~4 TB | 32,768 |

## Limits

- **Max filename**: 255 bytes
- **Max file size**: ~4 TB (4KB blocks)
- **Max filesystem**: 32 TB
- **Max inodes**: 4 billion
- **Max blocks**: 4 billion

## Common Patterns

### Check if File Exists

```rust
if root.lookup("file.txt").is_ok() {
    // File exists
}
```

### Get File Size

```rust
let size = root.lookup("file.txt")?.getattr()?.size;
```

### Append to File

```rust
let file = root.lookup("file.txt")?;
let attr = file.getattr()?;
file.write(attr.size, data)?;
```

### Copy File

```rust
let src = root.lookup("source.txt")?;
let data = {
    let attr = src.getattr()?;
    let mut buf = vec![0u8; attr.size as usize];
    src.read(0, &mut buf)?;
    buf
};

let dst = root.create("dest.txt", FileMode::new(0o644))?;
dst.write(0, &data)?;
```

### Walk Directory Tree

```rust
fn walk(dir: &Arc<dyn VNode>, depth: usize) -> Result<(), FsError> {
    for entry in dir.readdir()? {
        if entry.name == "." || entry.name == ".." {
            continue;
        }
        
        let child = dir.lookup(&entry.name)?;
        let attr = child.getattr()?;
        
        if matches!(attr.file_type, FileType::Directory) {
            walk(&child, depth + 1)?;
        }
    }
    Ok(())
}
```

## Debugging

### Check Filesystem State

```rust
let stats = fs.statfs()?;
println!("Blocks: {}/{}", 
         stats.blocks - stats.blocks_free, 
         stats.blocks);
println!("Inodes: {}/{}", 
         stats.files - stats.files_free, 
         stats.files);
```

### Verify Magic Number

```rust
use rinux_fs::ext2::detect_ext2;

if detect_ext2(&device)? {
    println!("Valid ext2 filesystem");
}
```

## Safety Notes

- Always sync before unmounting
- Check return values for errors
- Don't assume operations succeed
- Handle NoSpaceLeft gracefully
- Validate user input for paths

## Integration

### With AHCI Driver

```rust
// Initialize AHCI
rinux_block::ahci::init();

// Get first SATA device
let device = rinux_block::get_device(0)?;

// Mount ext2
let fs = Ext2Filesystem::mount(device)?;
```

### With Mount System

```rust
use rinux_fs::mount::{mount, MountFlags};

// Mount at specific path
mount("/mnt/disk", fs.clone(), MountFlags::new())?;

// Access via mount point
let mounted = rinux_fs::mount::get_mount("/mnt/disk")?;
```

## See Also

- `EXT2_IMPLEMENTATION.md` - Detailed implementation guide
- `examples/ext2_usage.rs` - Usage examples
- `src/vfs.rs` - VFS interface documentation
