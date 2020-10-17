use std:: {
    self,
    ffi::OsStr,
    os::windows::ffi::OsStrExt
};

pub fn to_os_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
    .encode_wide()
    .chain(Some(0).into_iter())
    .collect::<Vec<_>>()
}