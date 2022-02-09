#[macro_export]
macro_rules! make_errors {
    (
        $name:ident,
        $kit:expr,
        $($keys: ident => $values: expr,)+
    ) => {
        pub enum $name {
            $($keys),+
        }

        impl $name {
            pub fn as_error(&self) -> maoer_cores::error::Error {
                use $name::*;
                match *self {
                    $( $keys => {
                        let i = $crate::error::make_errcode($kit as i32, $keys as u32, None);
                        maoer_cores::error::Error::new(i as i32, String::from($values))
                    },)+
                    
                    _ => maoer_cores::error::Error::new(0, String::from("iujjj"))
                }
            }
        }
    };
}

// macro_rules! build {
//     ($($body:tt)*) => {
//         as_item! {
//             enum Test { $($body)* }
//         }
//     };
// }

// macro_rules! as_item {
//     ($i:item) => { $i };
// }
