#![allow(non_camel_case_types)]

struct mpc_state_t {
    pos: usize,
    row: usize,
    col: usize,
    term: i32,
}

struct mpc_err_t<'a> {
    state: mpc_state_t,
    expected_num: i32,
    filename: &'a str,
    failure: &'a str,
    expected: Vec<&'a str>,
    received: char,
}

enum mpc_result_t<'a> {
    Output(()),
    Err(mpc_err_t<'a>),
}
