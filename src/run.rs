pub fn cross_shell(cmd: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec!("cmd", "/C", cmd)
    } else {
        vec!("sh", "-c", cmd)
    }
    .iter().map(
        |x| x.to_string()
    ).collect()
}
