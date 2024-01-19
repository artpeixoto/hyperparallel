pub trait FnMutCallable {
    fn call(&mut self );
}

impl<TFn> FnMutCallable for TFn where TFn: FnMut(){
    fn call(&mut self, ) {
        (self)()
    }
}

impl FnMutCallable for (){
    fn call(&mut self) {}
}

struct FnMutList<TFnRest: FnMutCallable> (TFnRest);

impl FnMutList<()> {
    fn new() -> Self{ Self(()) }
}

impl<TFnRest: FnMutCallable> FnMutList<TFnRest> {
    fn push<TFnNew: FnMut()>(self, elem: TFnNew) -> FnMutList<(TFnNew, TFnRest)>{
        FnMutList((elem, self.0))
    }
}

impl<TFnHead, TFnRest>
    FnMutCallable for (TFnHead, TFnRest)
    where
        TFnHead: FnMut(),
        TFnRest: FnMutCallable,
{
    fn call(&mut self){
        (self.0)();
        (self.1).call();
    }
}






