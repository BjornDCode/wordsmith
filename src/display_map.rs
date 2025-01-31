// #[derive(Debug, Clone)]
// pub struct DisplayMap {
//     hidden: Vec<HiddenDisplayMapRange>,
//     headlines: Vec<HeadingLine>,
// }

// impl DisplayMap {
//     pub fn new() -> DisplayMap {
//         return DisplayMap {
//             hidden: vec![],
//             headlines: vec![],
//         };
//     }

//     pub fn get_removed_count(&self) -> usize {
//         let removed_count: usize = &self
//             .hidden
//             .iter()
//             .map(|range| range.start + range.length)
//             .sum();

//         return removed_count;
//     }

//     pub fn push_hidden_range(&mut self, start: usize, length: usize) {
//         &self.hidden.push(HiddenDisplayMapRange { start, length });
//     }

//     pub fn push_headline(&mut self, line_index: usize, level: usize) {
//         &self.headlines.push(HeadingLine { line_index, level });
//     }
// }

// #[derive(Debug, Clone, Copy)]
// struct HiddenDisplayMapRange {
//     start: usize,
//     length: usize,
// }

// #[derive(Debug, Clone, Copy)]
// struct HeadingLine {
//     line_index: usize,
//     level: usize,
// }
