#![feature(stmt_expr_attributes)]

/// Compute the number of bytes a field is offset from the address of an object.
/// # Examples
///
/// ```
/// # #![feature(stmt_expr_attributes)]
/// # #[macro_use] extern crate simple_field_offset;
///
/// struct Employee {
///     active: bool,
///     name: String,
///     favorite_color: [u8; 3]
/// }
///
/// # fn main() {
///     // Prints a number, value is dependent on the struct layout.
///     println!("{}", field_offset!(Employee, favorite_color));
///
///     // You can specify the resulting type as follows. The default is usize.
///     println!("{:?}", field_offset!(Employee, name, isize));
///
///     // You can compute the offsets of multiple fields like so.
///     let (active_offset, name_offset) = field_offset!(Employee, (active, name));
/// # }
/// ```
// This is how the computation works.
//
// unsafe {
//     // Pretend there is an object of $Type at address 0.
//     let fake_obj_ptr = 0 as *const $Struct;

//     // Obtain a pointer to a field in fake.
//     let fake_field_ptr = &(*fake_obj_ptr).$field as *const _;

//     // Compute the offset and return it as whatever type it is required as.
//     (fake_field_ptr as usize - fake_obj_ptr as usize) as $Offset
// }
// We can leave off the subtraction because we set fake_obj_ptr to 0.

#[macro_export]
macro_rules! field_offset {
    ($Struct:ty, $field:ident) => (
        field_offset!($Struct, $field, usize)
    );
    ($Struct:ty, $field:ident, $Offset:ty) => {
        (|| {
            unsafe {
                &(*(0 as *const $Struct)).$field as *const _ as $Offset
            }
        })()
    };
    ($Struct:ty, ( $($field:ident),+ )) => (
        (
            $( field_offset!($Struct, $field) ),+
        )
    );
    ($Struct:ty, ( $($field:ident),+ ), $Offset:ty ) => (
        (
            $( field_offset!($Struct, $field, $Offset) ),+
        )
    );
}

#[cfg(test)]
mod tests {
    struct Employee {
        active: bool,
        name: String,
        favorite_color: [u8; 3],
    }

    #[test]
    fn computes_correct_offset() {
        let favorite_color_offset = field_offset!(Employee, favorite_color);

        let employee = Employee {
            active: false,
            name: String::from("Mick"),
            favorite_color: [255, 0, 255],
        };

        let employee_ptr = &employee as *const Employee as usize;

        unsafe {
            assert_eq!(
                *((employee_ptr + favorite_color_offset) as *const [u8; 3]),
                [255, 0, 255]
            );
        }
    }

    #[test]
    fn compute_multiple_offsets() {
        let (active_offset, name_offset, favorite_color_offset) =
            field_offset!(Employee, (active, name, favorite_color));

        let employee = Employee {
            active: false,
            name: String::from("Mick"),
            favorite_color: [255, 0, 255],
        };

        let employee_ptr = &employee as *const Employee as usize;

        unsafe {
            assert_eq!(*((employee_ptr + active_offset) as *const bool), false);
            assert_eq!(
                *((employee_ptr + name_offset) as *const String),
                String::from("Mick")
            );
            assert_eq!(
                *((employee_ptr + favorite_color_offset) as *const [u8; 3]),
                [255, 0, 255]
            );
        }
    }

    #[test]
    fn allows_specifying_the_resulting_type_for_single_fields() {
        let _: isize = field_offset!(Employee, active, isize);
    }

    #[test]
    fn allows_specifying_the_resulting_type_for_multiple_fields() {
        let _: (isize, isize) = field_offset!(Employee, (active, name), isize);
    }
}
