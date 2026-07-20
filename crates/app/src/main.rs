use anon_sum_types_lib::{Result2, ResultErrorHolderExt};
use anon_sum_types_lib::{context, create_error, create_test_enum};

fn main() {
    let res = my_test();
    if let Err(e) = res {
        eprintln!("{:?}", e);
    }
}

fn my_test() -> Result2<String, MainError> {
    failing_fn_1().conv()?;
    failing_fn_2().conv()?;
    failing_fn_3()
        .map_err_inner(|e| match e {
            SubError3::MyNewError3(MyNewError3(err)) => MyNewError2(err),
        })
        .context(context!("my_test context"))?;
    Ok("my_test".to_string())
}

fn failing_fn_1() -> Result2<String, SubError1> {
    Ok("failing_fn_1".to_string())
}

fn failing_fn_2() -> Result2<String, SubError2> {
    Ok("failing_fn_2".to_string())
}

fn failing_fn_3() -> Result2<String, SubError3> {
    create_error!(MyNewError3("failing_fn_3".to_string()))
        .context(context!("failing_fn_3 context"))?;
    Ok("failing_fn_3".to_string())
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct MyNewError1(String);
#[derive(Debug, PartialEq, Eq, Clone)]
struct MyNewError2(String);
#[derive(Debug, PartialEq, Eq, Clone)]
struct MyNewError3(String);

create_test_enum!(MainError, MyNewError1, MyNewError2);
create_test_enum!(MainError2, MyNewError1, MyNewError2);
create_test_enum!(SubError1, MyNewError1);
create_test_enum!(SubError2, MyNewError2);
create_test_enum!(SubError3, MyNewError3);

SubError1_mapper_between!(MainError);
SubError2_mapper_between!(MainError);

SubError1_mapper_between!(MainError2);
