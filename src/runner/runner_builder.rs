
pub struct Runner{
    processes: Vec<Box<dyn FnMut()>>,

}

pub struct OuterInterleaver<TIter:Iterator>{
    iters:       Vec<TIter>,
    cur_index:   usize,
    nones_count: usize,
    finished:    bool,
}

impl<TIter: Iterator> OuterInterleaver<TIter> {
    pub fn new(iterators: Vec<TIter>) -> Self{
        OuterInterleaver{
            iters: iterators,
            cur_index: 0,
            nones_count: 0,
            finished: false,
        }
    }
}



impl<TIter: Iterator> Iterator for OuterInterleaver<TIter>{
    type Item = Option<TIter::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.finished {
            let count = self.iters.iter().count();
            let current = self.iters.get_mut(self.cur_index).unwrap().next();
            self.cur_index = (self.cur_index + 1) % count;
            if current.is_none(){
                self.nones_count = self.nones_count + 1 ;
            }
            if self.cur_index == 0{
                if self.nones_count == count {
                    self.finished = true;
                } else {
                    self.nones_count = 0;
                }
            }
            Some(current)
        } else {
            None
        }
    }
}

#[test]
fn simple_test(){
    let iters = vec![
        (0..3).into_iter(),
        (0..2).into_iter(),
        (0..1).into_iter(),
    ];
    let interleaver_res  = OuterInterleaver::new(iters).collect_vec();
    println!("{:#?}", &interleaver_res);
}