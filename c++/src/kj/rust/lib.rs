#[cxx::bridge(namespace = "kj")]
mod common {
    unsafe extern "C++" {}
}
/*
#[cxx::bridge(namespace = "kj")]
mod main {
    unsafe extern "C++" {
        include!("main.h");

        type ProcessContext;
    }
}
*/
