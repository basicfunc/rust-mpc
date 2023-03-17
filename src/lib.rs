#![allow(non_camel_case_types)]
use std::default;
use std::fmt::Arguments;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::raw::*;

#[derive(Debug, Clone, Copy, Default)]
struct mpc_state_t {
    pos: i32,
    row: i32,
    col: i32,
    term: i32,
}

// State Types

fn mpc_state_invalid() -> mpc_state_t {
    mpc_state_t {
        pos: -1,
        row: -1,
        col: -1,
        term: 0,
    }
}

fn mpc_state_new() -> mpc_state_t {
    mpc_state_t {
        pos: 0,
        row: 0,
        col: 0,
        term: 0,
    }
}

// Input Type
const MPC_INPUT_STRING: usize = 0;
const MPC_INPUT_FILE: usize = 1;
const MPC_INPUT_PIPE: usize = 2;

const MPC_INPUT_MARKS_MIN: usize = 32;

const MPC_INPUT_MEM_NUM: usize = 512;

struct mpc_mem_t {
    mem: [char; 64],
}

struct mpc_input_t {
    itype: usize,
    filename: String,
    state: mpc_state_t,

    string: String,
    buffer: Vec<char>,
    file: Option<File>,

    suppress: bool,
    backtrack: bool,
    marks_slot: i32,
    marks_num: i32,
    marks: Vec<mpc_state_t>,

    lasts: Vec<char>,
    last: char,

    mem_index: usize,
    mem_full: Vec<char>,
    mem: Vec<mpc_mem_t>,
}

fn mpc_input_new_string(filename: &str, string: &str) -> mpc_input_t {
    let itype = MPC_INPUT_STRING;
    let filename = filename.to_owned();
    let state = mpc_state_new();

    let string = string.to_owned();
    let buffer = vec![];
    let file = None;

    let suppress = false;
    let backtrack = false;
    let marks_num = 0;
    let marks_slot = MPC_INPUT_MARKS_MIN as i32;
    let marks: Vec<mpc_state_t> = Vec::with_capacity(marks_slot as usize);

    let lasts: Vec<char> = Vec::with_capacity(marks_slot as usize);
    let last = '\0';

    let mem_index = 0;
    let mem_full: Vec<char> = vec!['\0'; MPC_INPUT_MEM_NUM];
    let mem = Vec::with_capacity(MPC_INPUT_MEM_NUM);

    mpc_input_t {
        itype,
        filename,
        state,
        string,
        buffer,
        file,
        suppress,
        backtrack,
        marks_slot,
        marks_num,
        marks,
        lasts,
        last,
        mem_index,
        mem_full,
        mem,
    }
}

fn mpc_input_new_nstring(filename: &str, string: &str, length: usize) -> mpc_input_t {
    let itype = MPC_INPUT_STRING;
    let filename = filename.to_owned();
    let state = mpc_state_new();

    let string = string[..length].to_owned();
    let buffer = vec![];
    let file = None;

    let suppress = false;
    let backtrack = false;
    let marks_num = 0;
    let marks_slot = MPC_INPUT_MARKS_MIN as i32;
    let marks: Vec<mpc_state_t> = Vec::with_capacity(marks_slot as usize);

    let lasts: Vec<char> = Vec::with_capacity(marks_slot as usize);
    let last = '\0';

    let mem_index = 0;
    let mem_full: Vec<char> = vec!['\0'; MPC_INPUT_MEM_NUM];
    let mem = Vec::with_capacity(MPC_INPUT_MEM_NUM);

    mpc_input_t {
        itype,
        filename,
        state,
        string,
        buffer,
        file,
        suppress,
        backtrack,
        marks_slot,
        marks_num,
        marks,
        lasts,
        last,
        mem_index,
        mem_full,
        mem,
    }
}

fn mpc_input_new_pipe(filename: &str, pipe: File) -> mpc_input_t {
    let itype = MPC_INPUT_PIPE;
    let filename = filename.to_owned();
    let state = mpc_state_new();

    let string = String::new();
    let buffer = vec![];
    let file: Option<File> = Some(pipe);

    let suppress = false;
    let backtrack = false;
    let marks_num = 0;
    let marks_slot = MPC_INPUT_MARKS_MIN as i32;
    let marks: Vec<mpc_state_t> = Vec::with_capacity(marks_slot as usize);

    let lasts: Vec<char> = Vec::with_capacity(marks_slot as usize);
    let last = '\0';

    let mem_index = 0;
    let mem_full: Vec<char> = vec!['\0'; MPC_INPUT_MEM_NUM];
    let mem = Vec::with_capacity(MPC_INPUT_MEM_NUM);

    mpc_input_t {
        itype,
        filename,
        state,
        string,
        buffer,
        file,
        suppress,
        backtrack,
        marks_slot,
        marks_num,
        marks,
        lasts,
        last,
        mem_index,
        mem_full,
        mem,
    }
}

fn mpc_input_new_file(filename: &str, file: File) -> mpc_input_t {
    let itype = MPC_INPUT_STRING;
    let filename = filename.to_owned();
    let state = mpc_state_new();

    let string = String::new();
    let buffer = vec![];
    let file = Some(file);

    let suppress = false;
    let backtrack = false;
    let marks_num = 0;
    let marks_slot = MPC_INPUT_MARKS_MIN as i32;
    let marks: Vec<mpc_state_t> = Vec::with_capacity(marks_slot as usize);

    let lasts: Vec<char> = Vec::with_capacity(marks_slot as usize);
    let last = '\0';

    let mem_index = 0;
    let mem_full: Vec<char> = vec!['\0'; MPC_INPUT_MEM_NUM as usize];
    let mem = Vec::with_capacity(MPC_INPUT_MEM_NUM as usize);

    mpc_input_t {
        itype,
        filename,
        state,
        string,
        buffer,
        file,
        suppress,
        backtrack,
        marks_slot,
        marks_num,
        marks,
        lasts,
        last,
        mem_index,
        mem_full,
        mem,
    }
}

fn mpc_input_delete(i: &mut mpc_input_t) {
    drop(i)
}

fn is_ptr_inside<T>(p: *const T, q: &Vec<T>) -> bool {
    let q_start = q.as_ptr() as usize;
    let q_end = q_start + q.len() * std::mem::size_of::<T>();
    let p_addr = p as usize;
    p_addr >= q_start && p_addr < q_end
}

// TODO: Implemented Naively, more work needed.
fn mpc_mem_ptr(i: &mpc_input_t, p: *const u8) -> bool {
    let start = i.mem.as_ptr() as *const u8;
    let end = unsafe { start.add(MPC_INPUT_MEM_NUM * std::mem::size_of::<mpc_mem_t>()) };
    p >= start && p < end
}

// mpc_malloc, mpc_calloc, mpc_realloc and mpc_free are skipped as they don't need to implemented because Rust handle them gracefully.

// TODO: Implemented Naively, more work needed.
fn mpc_export(i: &mut mpc_input_t, p: *const u8) {
    if mpc_mem_ptr(i, p) {
        return;
    }
    unimplemented!()
}

fn mpc_input_backtrack_disable(i: &mut mpc_input_t) {
    i.backtrack = false;
}

fn mpc_input_backtrack_enable(i: &mut mpc_input_t) {
    i.backtrack = true;
}

fn mpc_input_suppress_disable(i: &mut mpc_input_t) {
    i.suppress = false;
}

fn mpc_input_suppress_enable(i: &mut mpc_input_t) {
    i.suppress = true;
}

fn mpc_input_mark(i: &mut mpc_input_t) {
    if !i.backtrack {
        return;
    }

    i.marks_num += 1;

    if i.marks_num > i.marks_slot {
        i.marks_slot = i.marks_num + i.marks_num / 2;
        i.marks.resize(i.marks_slot as usize, Default::default());
        i.lasts.resize(i.marks_slot as usize, Default::default());
    }

    i.marks[(i.marks_num - 1) as usize] = i.state;
    i.lasts[(i.marks_num - 1) as usize] = i.last;

    if i.itype == MPC_INPUT_PIPE && i.marks_num == 1 {
        i.buffer.resize(1, Default::default())
    }
}

fn mpc_input_unmark(i: &mut mpc_input_t) {
    if !i.backtrack {
        return;
    }

    if i.marks_slot > i.marks_num + i.marks_num / 2 && i.marks_slot > MPC_INPUT_MARKS_MIN as i32 {
        i.marks_slot = if i.marks_num > MPC_INPUT_MARKS_MIN as i32 {
            i.marks_num
        } else {
            MPC_INPUT_MARKS_MIN as i32
        }
    }

    if i.itype == MPC_INPUT_PIPE && i.marks_num == 0 {
        if i.file.is_none() {
            panic!("Error: No File Opened.")
        } else {
            let f = i.file.as_mut().unwrap();
            for idx in (0..i.buffer.len()).rev() {
                match write!(f, "{}", i.buffer[idx]) {
                    Ok(_) => (),
                    Err(e) => panic!("Error: {e} occured."),
                }
            }

            i.buffer.clear();
        }
    }
}

fn mpc_input_rewind(i: &mut mpc_input_t) {
    if !i.backtrack {
        return;
    }

    i.state = i.marks[(i.marks_num - 1) as usize];
    i.last = i.lasts[(i.marks_num - 1) as usize];

    if i.itype == MPC_INPUT_FILE {
        i.file
            .as_ref()
            .unwrap()
            .seek(SeekFrom::Start(i.state.pos as u64))
            .unwrap();
    }

    mpc_input_mark(i);
}

fn mpc_input_buffer_in_range(i: &mpc_input_t) -> bool {
    i.state.pos < (i.buffer.len() as i32 + i.marks[0].pos)
}

fn mpc_input_buffer_get(i: &mpc_input_t) -> char {
    i.buffer[(i.state.pos - i.marks[0].pos) as usize]
}

fn mpc_input_getc(i: &mpc_input_t) -> char {
    if i.itype == MPC_INPUT_STRING {
        return i.string.chars().nth(i.state.pos as usize).unwrap();
    } else if i.itype == MPC_INPUT_FILE {
        let mut buf = [0u8; 1];
        i.file.as_ref().unwrap().read(&mut buf).unwrap();
        return buf[0] as char;
    } else if i.itype == MPC_INPUT_PIPE {
        if i.buffer.len() > 0 && mpc_input_buffer_in_range(i) {
            return mpc_input_buffer_get(i);
        } else {
            let mut buf = [0u8; 1];
            i.file.as_ref().unwrap().read(&mut buf).unwrap();
            return buf[0] as char;
        }
    } else {
        return '\0';
    }
}

fn mpc_input_peekc(i: &mpc_input_t) -> char {
    if i.itype == MPC_INPUT_STRING {
        return i.string.chars().nth(i.state.pos as usize).unwrap();
    } else if i.itype == MPC_INPUT_FILE {
        let mut buf = [0u8; 1];
        let bytes_read = i.file.as_ref().unwrap().read(&mut buf).unwrap();

        if bytes_read == 0 {
            return '\0';
        }

        i.file
            .as_ref()
            .unwrap()
            .seek(SeekFrom::Current(-1))
            .unwrap();

        return buf[0] as char;
    } else if i.itype == MPC_INPUT_PIPE {
        if i.buffer.len() > 0 && mpc_input_buffer_in_range(i) {
            return mpc_input_buffer_get(i);
        } else {
            let mut buf = [0u8; 1];
            let bytes_read = i.file.as_ref().unwrap().read(&mut buf).unwrap();

            if bytes_read == 0 {
                return '\0';
            }
            let c = buf[0] as char;
            write!(i.file.as_ref().unwrap(), "{c}").unwrap();
            return c;
        }
    } else {
        return '\0';
    }
}

fn mpc_input_terminated(i: &mpc_input_t) -> bool {
    mpc_input_peekc(i) == '\0'
}

fn mpc_input_failure(i: &mpc_input_t, ch: char) {
    match i.itype {
        MPC_INPUT_FILE => {
            i.file
                .as_ref()
                .unwrap()
                .seek(SeekFrom::Current(-1))
                .unwrap();
        }
        MPC_INPUT_PIPE => {
            if i.buffer.is_empty() || (i.buffer.len() > 0 && !mpc_input_buffer_in_range(i)) {
                write!(i.file.as_ref().unwrap(), "{ch}").unwrap();
            }
        }
        _ => (),
    }
}

fn mpc_input_success(i: &mut mpc_input_t, c: char, o: Vec<&str>) {
    if i.itype == MPC_INPUT_PIPE && !i.buffer.is_empty() && !mpc_input_buffer_in_range(i) {
        let s = i.buffer.len();
        i.buffer.resize(s + 2, Default::default());
        let s = i.buffer.len();
        i.buffer[s + 1] = '\0';
        i.buffer[s + 0] = c;
    }

    i.last = c;
    i.state.pos += 1;
    i.state.col += 1;

    if c == '\n' {
        i.state.col = 0;
        i.state.row += 1;
    }

    if !o.is_empty() {
        // TODO
        unimplemented!()
    }
}

// Error Type
struct mpc_err_t {
    state: mpc_state_t,
    expected_num: i32,
    filename: String,
    failure: String,
    expected: Vec<String>,
    received: char,
}

// Related Functions
fn mpc_err_delete(e: &mpc_err_t) {}
fn mpc_err_string(e: &mpc_err_t) -> String {
    unimplemented!()
}
fn mpc_err_print(e: &mpc_err_t) {}
fn mpc_err_print_to(e: &mpc_err_t, f: &mut std::fs::File) {}

// Parsing
type mpc_val_t = c_void;
type mpc_result_t = Result<Box<mpc_val_t>, mpc_err_t>;

struct mpc_parser_t;

// Related Functions
fn mpc_parse(filename: &str, string: &str, p: &mpc_parser_t, r: &mpc_result_t) -> i32 {
    unimplemented!()
}

fn mpc_nparse(
    filename: &str,
    string: &str,
    length: usize,
    p: &mpc_parser_t,
    r: &mpc_result_t,
) -> i32 {
    unimplemented!()
}

fn mpc_parse_file(filename: &str, file: &mut File, p: &mpc_parser_t, r: &mpc_result_t) -> i32 {
    unimplemented!()
}
fn mpc_parse_pipe(filename: &str, pipe: &mut File, p: &mpc_parser_t, r: &mpc_result_t) -> i32 {
    unimplemented!()
}

fn mpc_parse_contents(filename: &str, p: &mpc_parser_t, r: &mpc_result_t) -> i32 {
    unimplemented!()
}

// Function Types

type mpc_dtor_t = fn(&mpc_val_t);
type mpc_ctor_t = fn() -> mpc_val_t;

type mpc_apply_t = fn(&mpc_val_t) -> mpc_val_t;
type mpc_apply_to_t = fn(&mpc_val_t, &c_void) -> mpc_val_t;
type mpc_fold_t = fn(&&mpc_val_t, isize) -> mpc_val_t;

type mpc_check_t = fn(&&mpc_val_t) -> isize;
type mpc_check_with_t = fn(&&mpc_val_t, &c_void) -> isize;

// Building a Parser
fn mpc_new(name: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_copy(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_define(p: &mpc_parser_t, a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_undefine(p: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_delete(p: &mpc_parser_t) {}
fn mpc_cleanup(n: isize, args: Arguments) {}

// Basic Parsers
fn mpc_any() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_char(c: char) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_range(s: char, e: char) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_oneof(s: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_noneof(s: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_satisfy(f: fn(char) -> i32) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_string(s: &str) -> mpc_parser_t {
    unimplemented!()
}

// Other Parsers

fn mpc_pass() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_fail(m: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_failf(fmt: &str, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_lift(f: mpc_ctor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_lift_val(x: Box<mpc_val_t>) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_anchor(f: fn(char, char) -> i32) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_state() -> mpc_state_t {
    unimplemented!()
}

// Combinator Parsers
fn mpc_expect(a: &mpc_parser_t, e: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_expectf(a: &mpc_parser_t, fmt: &str, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_apply(a: &mpc_parser_t, f: mpc_apply_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_apply_to(a: &mpc_parser_t, f: mpc_apply_to_t, x: Box<c_void>) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_check(a: &mpc_parser_t, da: mpc_dtor_t, f: mpc_check_t, e: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_check_with(
    a: &mpc_parser_t,
    da: mpc_dtor_t,
    f: mpc_check_with_t,
    x: Box<c_void>,
    e: &str,
) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_checkf(
    a: &mpc_parser_t,
    da: mpc_dtor_t,
    f: mpc_check_t,
    fmt: &str,
    args: Arguments,
) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_check_withf(
    a: &mpc_parser_t,
    da: mpc_dtor_t,
    f: mpc_check_with_t,
    x: Box<c_void>,
    fmt: &str,
    args: Arguments,
) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_not(a: &mpc_parser_t, da: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_not_lift(a: &mpc_parser_t, da: mpc_dtor_t, lf: mpc_ctor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_maybe(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_maybe_lift(a: &mpc_parser_t, lf: mpc_ctor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_many(f: mpc_fold_t, a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_many1(f: mpc_fold_t, a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_count(n: i32, f: mpc_fold_t, a: &mpc_parser_t, da: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_or(n: i32, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_and(n: i32, f: mpc_fold_t, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_predictive(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

// Common Parsers

fn mpc_eoi() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_soi() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_boundary() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_boundary_newline() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_whitespace() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_whitespaces() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_blank() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_digit() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_hexdigit() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_octdigit() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_digits() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_hexdigits() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_octdigits() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_lower() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_upper() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_alpha() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_underscore() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_alphanum() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_int() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_hex() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_oct() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_number() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_real() -> mpc_parser_t {
    unimplemented!()
}
fn mpc_float() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_char_lit() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_string_lit() -> mpc_parser_t {
    unimplemented!()
}

fn mpc_regex_lit() -> mpc_parser_t {
    unimplemented!()
}

// Useful Parsers

fn mpc_startwith(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_endwith(a: &mpc_parser_t, da: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_whole(a: &mpc_parser_t, da: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_stripl(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_stripr(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_strip(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_tok(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_sym(s: &str) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_total(a: &mpc_parser_t, da: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_between(a: &mpc_parser_t, ad: mpc_dtor_t, o: &str, c: &str) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_parens(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_braces(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_brackets(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_squares(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_tok_between(a: &mpc_parser_t, ad: mpc_dtor_t, o: &str, c: &str) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_tok_parens(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_tok_braces(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_tok_brackets(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpc_tok_squares(a: &mpc_parser_t, ad: mpc_dtor_t) -> mpc_parser_t {
    unimplemented!()
}

// Common Function Parameters

fn mpcf_dtor_null(x: Box<mpc_val_t>) {}

fn mpcf_ctor_null() -> mpc_val_t {
    unimplemented!()
}
fn mpcf_ctor_str() -> mpc_val_t {
    unimplemented!()
}

fn mpcf_free(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_int(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_hex(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_oct(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_float(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_strtriml(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_strtrimr(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_strtrim(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}

fn mpcf_escape(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_escape_regex(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_escape_string_raw(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_escape_char_raw(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}

fn mpcf_unescape(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_unescape_regex(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_unescape_string_raw(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_unescape_char_raw(x: Box<mpc_val_t>) -> mpc_val_t {
    unimplemented!()
}

fn mpcf_null(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_fst(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_snd(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_trd(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}

fn mpcf_all_free(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_fst_free(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_snd_free(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_trd_free(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}

fn mpcf_freefold(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_strfold(n: i32, xs: Box<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}

// Regular Expression Parsers

enum MPC {
    RE_DEFAULT,
    RE_M,
    RE_S,
    RE_MULTILINE,
    RE_DOTALL,
}

fn mpc_re(re: &str) -> mpc_parser_t {
    unimplemented!()
}

fn mpc_re_mode(re: &str, mode: i32) -> mpc_parser_t {
    unimplemented!()
}

// AST
struct mpc_ast_t {
    tag: String,
    contents: String,
    state: mpc_state_t,
    children_num: isize,
    children: Vec<Option<Box<mpc_ast_t>>>,
}

fn mpc_ast_new(tag: &str, contents: &str) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_build(n: i32, tag: &str, args: Arguments) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_add_root(a: &mpc_ast_t) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_add_child(r: &mpc_ast_t, a: &mpc_ast_t) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_add_tag(a: &mpc_ast_t, t: &str) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_add_root_tag(a: &mpc_ast_t, t: &str) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_tag(a: &mpc_ast_t, t: &str) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_state(a: &mpc_ast_t, s: mpc_state_t) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_delete(a: &mpc_ast_t) {}
fn mpc_ast_print(a: &mpc_ast_t) {}
fn mpc_ast_print_to(a: &mpc_ast_t, f: &mut File) {}

fn mpc_ast_get_index(ast: &mpc_ast_t, tag: &str) -> i32 {
    unimplemented!()
}
fn mpc_ast_get_index_lb(ast: &mpc_ast_t, tag: &str, lb: i32) -> i32 {
    unimplemented!()
}

fn mpc_ast_get_child(ast: &mpc_ast_t, tag: &str) -> mpc_ast_t {
    unimplemented!()
}
fn mpc_ast_get_child_lb(ast: &mpc_ast_t, tag: &str, lb: i32) -> mpc_ast_t {
    unimplemented!()
}

enum mpc_ast_trav_order_t {
    mpc_ast_trav_order_pre,
    mpc_ast_trav_order_post,
}

struct mpc_ast_trav_t {
    curr_node: Option<Box<mpc_ast_t>>,
    parent: Option<Box<mpc_ast_trav_t>>,
    curr_child: i32,
    order: mpc_ast_trav_order_t,
}

fn mpc_ast_traverse_start(ast: &mpc_ast_t, order: mpc_ast_trav_order_t) -> mpc_ast_trav_t {
    unimplemented!()
}

fn mpc_ast_traverse_next(trav: Vec<Option<Box<mpc_ast_trav_t>>>) -> mpc_ast_t {
    unimplemented!()
}

fn mpc_ast_traverse_free(trav: Vec<Option<Box<mpc_ast_trav_t>>>) {}

// Warning: This function currently doesn't test for equality of the `state` member!
fn mpc_ast_eq(a: mpc_ast_t, b: mpc_ast_t) -> i32 {
    unimplemented!()
}

fn mpcf_fold_ast(n: i32, xs: Vec<Option<Box<mpc_val_t>>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_str_ast(c: Option<Box<mpc_val_t>>) -> mpc_val_t {
    unimplemented!()
}
fn mpcf_state_ast(n: i32, xs: Vec<Option<Box<mpc_val_t>>>) -> mpc_val_t {
    unimplemented!()
}

fn mpca_tag(a: &mpc_parser_t, t: &str) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_add_tag(a: &mpc_parser_t, t: &str) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_root(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_state(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_total(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpca_not(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_maybe(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpca_many(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_many1(a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_count(n: i32, a: &mpc_parser_t) -> mpc_parser_t {
    unimplemented!()
}

fn mpca_or(n: i32, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_and(n: i32, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}

enum MPCA {
    LANG_DEFAULT = 0,
    LANG_PREDICTIVE = 1,
    LANG_WHITESPACE_SENSITIVE = 2,
}

fn mpca_grammer(flags: i32, grammer: &str, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_lang(flags: i32, language: &str, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_lang_file(flags: i32, f: &mut File, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_lang_pipe(flags: i32, f: &mut File, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}
fn mpca_lang_contents(flags: i32, filename: &str, args: Arguments) -> mpc_parser_t {
    unimplemented!()
}

// Misc

fn mpc_print(p: &mpc_parser_t) {}
fn mpc_optimise(p: &mpc_parser_t) {}
fn mpc_stats(p: &mpc_parser_t) {}

fn mpc_test_pass(
    p: &mpc_parser_t,
    s: &str,
    d: Option<Box<c_void>>,
    tester: fn(*const c_void, *const c_void) -> i32,
    destructor: mpc_dtor_t,
    printer: fn(*const c_void),
) -> i32 {
    unimplemented!()
}

fn mpc_test_fail(
    p: &mpc_parser_t,
    s: &str,
    d: Option<Box<c_void>>,
    tester: fn(*const c_void, *const c_void) -> i32,
    destructor: mpc_dtor_t,
    printer: fn(*const c_void),
) -> i32 {
    unimplemented!()
}
