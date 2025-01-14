use std::convert::TryInto;
use test_winrt::Windows::Foundation::{IStringable, PropertyValue, Uri};

#[test]
fn class() -> windows::Result<()> {
    let uri = Uri::CreateUri("http://kennykerr.ca")?;

    // All WinRT classes are convertible to windows::IInspectable.
    let object: windows::IInspectable = uri.into();

    assert!(object.type_name()? == "Windows.Foundation.Uri");

    Ok(())
}

#[test]
fn interface() -> windows::Result<()> {
    let uri = Uri::CreateUri("http://kennykerr.ca")?;
    let stringable: IStringable = uri.try_into().unwrap();

    // All WinRT interfaces are convertible to windows::IInspectable.
    let object: windows::IInspectable = stringable.into();

    assert!(object.type_name()? == "Windows.Foundation.Uri");

    Ok(())
}

#[test]
fn boxing() -> windows::Result<()> {
    let object = PropertyValue::CreateString("hello")?;

    assert!(object.type_name()? == "Windows.Foundation.IReference`1<String>");

    Ok(())
}
