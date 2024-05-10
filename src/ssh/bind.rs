// use std::os::raw::{c_int};
use std::ffi::{c_char, c_int};

extern "C" {
    /*
int ssh_connect_shell(const char *user, const char *password, const char *host, int port)
{
    printf("sshpass -p %s ssh -p %d %s@%s\n", password, port, user, host);
    return 1024;
}
    */
    pub fn ssh_connect_shell(user: *const c_char, password: *const c_char, host: *const c_char, port: c_int) -> c_int;
}