pub fn read_orders_from_excel() {
    println!("calamine does not support reading image from cell.")
}

// use calamine::{open_workbook, Reader, Xlsx};
//
// pub fn read_order_from_excel() {
//     let path = "./src/excel/order_template.xlsx";
//     // opens a new workbook
//     let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot open file");
//
//     let sheet_names = workbook.sheet_names();
//     println!("{:?}", sheet_names);
//
//     let range = workbook
//         .worksheet_range(sheet_names[0].clone().as_str())
//         .unwrap()
//         .unwrap();
//     let n = range.get_size().1 - 1;
//     for r in range.rows() {
//         for (i, c) in r.iter().enumerate() {
//             println!("{i}: {:?}", c);
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::excel::order_excel::read_order_from_excel;
//
//     #[test]
//     fn test_read_excel() {
//         read_order_from_excel()
//     }
// }
