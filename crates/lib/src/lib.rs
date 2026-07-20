use std::fmt::Debug;

pub type Result2<T, E> = Result<T, ErrorHolder<E>>;

#[derive(Debug, PartialEq, Eq, Clone)]
struct AccountId(u32);
#[derive(Debug, PartialEq, Eq, Clone)]
struct User(u32);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AccountNotFound {
    account_id: AccountId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserNotFound {
    user_id: User,
}

#[derive(PartialEq, Eq, Clone)]
pub struct ErrorHolder<T> {
    error: T,
    stack: Vec<StackTraceElementWithContext>,
}

impl <T> ErrorHolder<T> {
    pub fn new(error: T, stack: Vec<StackTraceElementWithContext>) -> Self {
        Self { error, stack }
    }

    pub fn error(&self) -> &T {
        &self.error
    }

    pub fn stack(&self) -> &Vec<StackTraceElementWithContext> {
        &self.stack
    }
}

impl<T: Debug> Debug for ErrorHolder<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        writeln!(f, "Error: {:?}", self.error)?;
        writeln!(f)?;

        if let Some(first_element) = self.stack.first() {
            writeln!(
                f,
                "thread 'null' (1) panicked at {}:{}:{}:",
                first_element
                    .stack_trace_element
                    .file_name
                    .replace("/", "\\"),
                first_element.stack_trace_element.line_number,
                first_element.stack_trace_element.column_number
            )?;
            writeln!(f, "stack backtrace:")?;
        }

        for (index, element) in self.stack.iter().enumerate() {
            writeln!(
                f,
                "   {}: {}",
                index, element.stack_trace_element.function_name
            )?;
            writeln!(
                f,
                "             at .\\{}:{}:{}",
                element.stack_trace_element.file_name,
                element.stack_trace_element.line_number,
                element.stack_trace_element.column_number
            )?;
            writeln!(f, "             context: {}", element.context)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StackTraceElement {
    function_name: &'static str,
    file_name: &'static str,
    line_number: u32,
    column_number: u32,
}

impl StackTraceElement {
    pub fn new(
        function_name: &'static str,
        file_name: &'static str,
        line_number: u32,
        column_number: u32,
    ) -> Self {
        Self {
            function_name,
            file_name,
            line_number,
            column_number,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackTraceElementWithContext {
    stack_trace_element: StackTraceElement,
    context: String,
}

impl StackTraceElementWithContext {
    pub fn new(stack_trace_element: StackTraceElement, context: String) -> Self {
        Self {
            stack_trace_element,
            context,
        }
    }
}

#[macro_export]
macro_rules! create_error {
    ($error:expr) => {{
        ::std::result::Result::Err(::anon_sum_types_lib::ErrorHolder::new(
            $error,
            vec![$crate::context!("INIT")],
        ))
    }};
}

pub fn create_error<T, E>(error: E, stack_trace_element_with_context: StackTraceElementWithContext) -> Result<T, ErrorHolder<E>> {
    let mut stack = Vec::new();
    stack.push(stack_trace_element_with_context);
    Err(ErrorHolder::new(error, stack))
}

#[macro_export]
macro_rules! create_stack_trace {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            ::std::any::type_name::<T>()
        }
        let function_name = type_name_of(f);
        let function_name = function_name.strip_suffix("::f").unwrap();
        $crate::StackTraceElement::new(function_name, file!(), line!(), column!())
    }};
}

#[macro_export]
macro_rules! context {
    ($($arg:tt)*) => {{
        let stack_trace_element = $crate::create_stack_trace!();
        let context = format!($($arg)*);
        $crate::StackTraceElementWithContext::new(stack_trace_element, context)
    }};
}

impl<T> ErrorHolder<T> {
    pub fn custom_into<U: From<T>>(self) -> ErrorHolder<U> {
        ErrorHolder {
            error: self.error.into(),
            stack: self.stack,
        }
    }

    pub fn map<U, F>(self, f: F) -> ErrorHolder<U>
    where
        F: FnOnce(T) -> U,
    {
        ErrorHolder {
            error: f(self.error),
            stack: self.stack,
        }
    }

    pub fn convert<U: From<T>>(self) -> ErrorHolder<U> {
        ErrorHolder {
            error: self.error.into(),
            stack: self.stack,
        }
    }
}

pub trait ResultErrorHolderExt<T, E> {
    fn conv<U: From<E>>(self) -> Result<T, ErrorHolder<U>>;
    fn context<U: From<E>>(
        self,
        context: StackTraceElementWithContext,
    ) -> Result<T, ErrorHolder<U>>;
    fn map_err_inner<U, F>(self, f: F) -> Result<T, ErrorHolder<U>>
    where
        F: FnOnce(E) -> U;
    fn to_either(self) -> Result<T, E>;
}

impl<T, E> ResultErrorHolderExt<T, E> for Result<T, ErrorHolder<E>> {
    fn conv<U: From<E>>(self) -> Result<T, ErrorHolder<U>> {
        self.map_err(ErrorHolder::convert)
    }

    fn context<U: From<E>>(
        self,
        context: StackTraceElementWithContext,
    ) -> Result<T, ErrorHolder<U>> {
        self.map_err(|mut e| {
            e.stack.push(context);
            e.convert()
        })
    }

    fn map_err_inner<U, F>(self, f: F) -> Result<T, ErrorHolder<U>>
    where
        F: FnOnce(E) -> U,
    {
        self.map_err(|e| e.map(f))
    }

    fn to_either(self) -> Result<T, E> {
        self.map_err(|e| e.error)
    }
}

#[macro_export]
macro_rules! create_test_enum {
    ($enum_name:ident, $($struct_name:ident),+ $(,)?) => {
        #[derive(::std::cmp::PartialEq, ::std::cmp::Eq, ::std::clone::Clone)]
        pub enum $enum_name {
            $(
                $struct_name($struct_name),
            )+
        }

        impl ::std::fmt::Debug for $enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $(
                        $enum_name::$struct_name(inner) => ::std::fmt::Debug::fmt(inner, f),
                    )+
                }
            }
        }

        $(
            impl ::std::convert::From<$struct_name> for $enum_name {
                fn from(value: $struct_name) -> Self {
                    $enum_name::$struct_name(value)
                }
            }
        )+

        ::paste::paste! {
            #[macro_export]
            macro_rules! [<$enum_name _mapper_between>] {
                ($target:ty) => {
                    impl ::std::convert::From<$enum_name> for $target {
                        fn from(value: $enum_name) -> Self {
                            match value {
                                $(
                                    $enum_name::$struct_name(inner) => <$target>::$struct_name(inner),
                                )+
                            }
                        }
                    }
                };
            }
        }
    };
}

create_test_enum!(TestEnum, UserNotFound);
create_test_enum!(TestEnum2, UserNotFound, AccountNotFound);

#[cfg(test)]
mod tests {
    use super::*;
    TestEnum_mapper_between!(TestEnum2);

    #[test]
    fn create_test_enum_wraps_struct_in_matching_variant() {
        let user_not_found = TestEnum::UserNotFound(UserNotFound { user_id: User(1) });

        match user_not_found {
            TestEnum::UserNotFound(UserNotFound {
                user_id: User(value),
            }) => assert_eq!(value, 1),
        }
    }

    #[test]
    fn create_test_enum_converts_structs_into_enum_variants() {
        let user_not_found: TestEnum = UserNotFound { user_id: User(7) }.into();

        match user_not_found {
            TestEnum::UserNotFound(UserNotFound {
                user_id: User(value),
            }) => assert_eq!(value, 7),
        }
    }

    #[test]
    fn mapper_implements_from_between_enums() {
        let test = TestEnum::UserNotFound(UserNotFound { user_id: User(42) });
        let result: TestEnum2 = test.into();
        match result {
            TestEnum2::UserNotFound(UserNotFound { user_id: User(v) }) => assert_eq!(v, 42),
            TestEnum2::AccountNotFound(_) => panic!("unexpected variant"),
        }
    }

    #[test]
    fn test_stack_trace_location() {
        let location = create_stack_trace!();
        let expected = StackTraceElement {
            function_name: "anon_sum_types_lib::tests::test_stack_trace_location",
            file_name: "crates\\lib\\src\\lib.rs",
            line_number: line!() - 4, // Adjusted to match the line number of the create_stack_trace! macro invocation
            column_number: 24,
        };
        assert_eq!(location, expected);
    }

    #[test]
    fn test_context() {
        let location = context!("{} {}", "hello", "world");
        let expected = StackTraceElementWithContext {
            stack_trace_element: StackTraceElement {
                function_name: "anon_sum_types_lib::tests::test_context",
                file_name: "crates\\lib\\src\\lib.rs",
                line_number: line!() - 5, // Adjusted to match the line number of the context! macro invocation
                column_number: 24,
            },
            context: "hello world".to_string(),
        };
        assert_eq!(location, expected);
    }

    #[test]
    fn debug_prints_nested_object_without_enum_name() {
        let user_not_found = TestEnum::UserNotFound(UserNotFound { user_id: User(42) });
        let debug_str = format!("{:?}", user_not_found);
        assert_eq!(debug_str, "UserNotFound { user_id: User(42) }");
    }
}
