use capnp_sys::call;
use eyre::Result;
use std::path::PathBuf;
use std::str::FromStr;

fn get_samples_dir() -> Result<PathBuf> {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()?
        .stdout;
    let workspace = PathBuf::from_str(std::str::from_utf8(&output)?)?;
    let parent = workspace.parent().expect("workspace has no parent?!");

    Ok(parent.join("c++").join("samples"))
}

#[should_panic]
#[test]
fn test_address_book() {
    let path = get_samples_dir().unwrap().join("addressbook.capnp");
    let _bytes = call(
        [path.to_string_lossy()].into_iter(),
        Vec::<String>::new().into_iter(),
        Vec::<String>::new().into_iter(),
        true,
    )
    .unwrap();
}

#[test]
fn test_calculator() -> Result<()> {
    let path = get_samples_dir()?.join("calculator.capnp");
    let bytes = call(
        [path.to_string_lossy()].into_iter(),
        Vec::<String>::new().into_iter(),
        Vec::<String>::new().into_iter(),
        true,
    )?;
    assert_ne!(bytes.len(), 0);
    Ok(())
}

#[test]
fn test_id() -> Result<()> {
    capnp_sys::id();
    Ok(())
}