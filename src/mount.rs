use std::fs;
use std::io;

fn get_rucker_container_path() -> String {
    String::from("/var/run/gocker/containers")
}

fn create_container_directories(container_id: &str) -> io::Result<()> {
    let container_home = get_rucker_container_path() + "/" + container_id;
    let container_dirs = vec![
        // FIXME: 流石にこの書き方は無いか…
        container_home.clone() + "/fs",
        container_home.clone() + "/fs/mnt",
        container_home.clone() + "/fs/upperdir",
        container_home.clone() + "/fs/workdir",
    ];
    for dir in container_dirs {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn aaa() {
        let result = super::create_container_directories("sample");
        assert_eq!(result.is_ok(), true);
        let expected_dir_path = super::get_rucker_container_path() + "/sample";
        assert_eq!(Path::new(&expected_dir_path).exists(), false);
    }
}
