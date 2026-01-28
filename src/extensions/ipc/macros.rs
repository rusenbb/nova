//! Helper macros for IPC operations.
//!
//! These macros reduce boilerplate when implementing Nova IPC operations.

/// Get the Nova context from OpState with immutable borrow.
///
/// # Example
/// ```ignore
/// fn my_op(state: &mut OpState) -> Result<(), AnyError> {
///     let ctx = nova_ctx!(state);
///     // use ctx...
/// }
/// ```
#[macro_export]
macro_rules! nova_ctx {
    ($state:expr) => {
        $state.borrow::<$crate::extensions::ipc::NovaContext>()
    };
}

/// Get the Nova context from OpState with mutable borrow.
///
/// # Example
/// ```ignore
/// fn my_op(state: &mut OpState) -> Result<(), AnyError> {
///     let ctx = nova_ctx_mut!(state);
///     ctx.should_close = true;
/// }
/// ```
#[macro_export]
macro_rules! nova_ctx_mut {
    ($state:expr) => {
        $state.borrow_mut::<$crate::extensions::ipc::NovaContext>()
    };
}

/// Check a permission and return early with error if not granted.
///
/// # Example
/// ```ignore
/// fn my_op(state: &mut OpState) -> Result<(), AnyError> {
///     let ctx = nova_ctx!(state);
///     nova_check_permission!(ctx, "clipboard");
///     // permission granted, proceed...
/// }
/// ```
#[macro_export]
macro_rules! nova_check_permission {
    ($ctx:expr, $perm:expr) => {
        $ctx.check_permission($perm)?
    };
}

/// Combined macro: get context and check permission in one call.
///
/// # Example
/// ```ignore
/// fn op_my_feature(state: &mut OpState) -> Result<String, AnyError> {
///     let ctx = nova_with_permission!(state, "system");
///     // ctx is ready to use with permission already verified
///     Ok(ctx.platform.get_something())
/// }
/// ```
#[macro_export]
macro_rules! nova_with_permission {
    ($state:expr, $perm:expr) => {{
        let ctx = $state.borrow::<$crate::extensions::ipc::NovaContext>();
        ctx.check_permission($perm)?;
        ctx
    }};
}

/// Combined macro for mutable context with permission check.
///
/// # Example
/// ```ignore
/// fn op_my_feature(state: &mut OpState) -> Result<(), AnyError> {
///     let ctx = nova_with_permission_mut!(state, "storage");
///     ctx.storage.set("key", value)?;
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! nova_with_permission_mut {
    ($state:expr, $perm:expr) => {{
        let ctx = $state.borrow_mut::<$crate::extensions::ipc::NovaContext>();
        ctx.check_permission($perm)?;
        ctx
    }};
}

pub use nova_check_permission;
pub use nova_ctx;
pub use nova_ctx_mut;
pub use nova_with_permission;
pub use nova_with_permission_mut;

#[cfg(test)]
mod tests {
    // Note: These macros require OpState and NovaContext which are complex to mock.
    // Testing is done via integration tests with the actual extension runtime.
}
