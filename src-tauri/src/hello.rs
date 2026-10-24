//! Windows Hello verification for locked Spaces.

use windows::{
    core::{factory, HSTRING},
    Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    },
    Win32::{Foundation::HWND, System::WinRT::IUserConsentVerifierInterop},
};
use windows_future::IAsyncOperation;

/// Whether this PC has Windows Hello (PIN, face or fingerprint) set up.
pub fn available() -> bool {
    UserConsentVerifier::CheckAvailabilityAsync()
        .and_then(|op| op.join())
        .is_ok_and(|a| a == UserConsentVerifierAvailability::Available)
}

/// Asks for Windows Hello over `hwnd`; blocks until the person answers.
pub fn verify(hwnd: isize, message: &str) -> bool {
    let run = || -> windows::core::Result<bool> {
        let interop = factory::<UserConsentVerifier, IUserConsentVerifierInterop>()?;
        let op: IAsyncOperation<UserConsentVerificationResult> = unsafe {
            interop.RequestVerificationForWindowAsync(HWND(hwnd as _), &HSTRING::from(message))?
        };
        Ok(op.join()? == UserConsentVerificationResult::Verified)
    };
    run().unwrap_or(false)
}
