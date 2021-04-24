use std::io::{Read, Write};

use quote::{quote, ToTokens};

fn catch_unwind_silent_in_proc_macro_error<
    F: FnOnce() -> R + std::panic::UnwindSafe,
    R: Send + 'static,
>(
    f: F,
) -> R {
    use proc_macro_error::entry_point;
    use std::{
        panic::{panic_any, resume_unwind, set_hook, take_hook},
        sync::Arc,
    };

    struct TestSuccess<R>(R);

    let prev_hook = Arc::new(take_hook());
    let prev_hook2 = prev_hook.clone();
    set_hook(Box::new(move |p| {
        if !p.payload().is::<TestSuccess<R>>() {
            prev_hook2(p)
        }
    }));
    let result = std::panic::catch_unwind(|| {
        entry_point(
            || {
                let r = f();
                panic_any(TestSuccess(r));
            },
            false,
        )
    });
    drop(take_hook());
    set_hook(
        Arc::try_unwrap(prev_hook).unwrap_or_else(|_| panic!("Unable to unwrap panic_handler")),
    );

    match result.unwrap_err().downcast::<TestSuccess<R>>() {
        Ok(x) => x.0,
        Err(z) => resume_unwind(z),
    }
}

fn rustfmt(code: &str) -> String {
    let mut child = std::process::Command::new("rustfmt")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();

    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();

    out
}

fn gitdiff(code: &str, file: &str) -> String {
    let mut child = std::process::Command::new("git")
        .arg("diff")
        .arg("--no-index")
        .arg("--")
        .arg(file)
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .take()
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();

    let mut out = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();

    out
}

#[test]
fn pass_in_macro() {
    catch_unwind_silent_in_proc_macro_error(|| {
        let out = rustfmt(
            &crate::gobject_signal_properties_impl(
                quote! {},
                std::str::FromStr::from_str(include_str!("pass_in_macro.rs.in")).unwrap(),
            )
            .to_string(),
        );
        if &out != include_str!("pass_in_macro.rs.out") {
            panic!(
                "{}",
                gitdiff(
                    &out,
                    &format!(
                        "{}/src/test/{}",
                        env!("CARGO_MANIFEST_DIR"),
                        "pass_in_macro.rs.out"
                    ),
                )
            );
        }
    });
    println!("Test running");
}
