use std::fs;
use std::io;
use std::path::Path;

pub fn write_pid_file(path: impl AsRef<Path>, pid: u32) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, pid.to_string())
}

pub fn read_pid_file(path: impl AsRef<Path>) -> io::Result<Option<u32>> {
    let path = path.as_ref();
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(io::Error::new(
                error.kind(),
                format!("读取 PID 文件失败: {error}"),
            ));
        }
    };

    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "PID 文件内容为空",
        ));
    }

    let pid = trimmed.parse::<u32>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("PID 文件内容不是有效数字: {error}"),
        )
    })?;
    Ok(Some(pid))
}

pub fn clear_pid_file(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io::Error::new(
            error.kind(),
            format!("清理 PID 文件失败: {error}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{clear_pid_file, read_pid_file, write_pid_file};
    use tempfile::tempdir;

    #[test]
    fn pid_file_round_trip_works() {
        let temp_dir = tempdir().expect("create temp dir");
        let pid_file = temp_dir.path().join("nested").join("gost.pid");

        write_pid_file(&pid_file, 12345).expect("write pid file");
        assert_eq!(
            read_pid_file(&pid_file).expect("read pid file"),
            Some(12345)
        );

        clear_pid_file(&pid_file).expect("clear pid file");
        assert_eq!(
            read_pid_file(&pid_file).expect("read cleared pid file"),
            None
        );
    }

    #[test]
    fn invalid_pid_file_content_is_rejected() {
        let temp_dir = tempdir().expect("create temp dir");
        let pid_file = temp_dir.path().join("gost.pid");
        std::fs::write(&pid_file, "not-a-pid").expect("write invalid pid file");

        let error = read_pid_file(&pid_file).expect_err("invalid pid should fail");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
