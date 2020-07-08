use std::fs;

fn get_rucker_container_path() -> String {
    String::from("/var/run/gocker/containers")
}

fn create_container_directories(container_id: &str) {
    let container_home = get_rucker_container_path() + "/" + container_id;
    let container_dirs = vec![
        container_home + "/fs",
        container_home + "/fs/mnt",
        container_home + "/fs/upperdir",
        contHome + "/fs/workdir",
    ];
    for dir in container_dirs {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        let result = super::create_container_directories("sample");
        // assert_eq!(result, Ok(()));
        assert_eq!(result, "hoge")
    }
}
