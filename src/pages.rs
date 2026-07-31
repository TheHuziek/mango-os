const PAGE_SIZE: usize = 4096;
struct page {
    prev: usize,
    next: usize,
    addr: usize,
}
impl page {}
