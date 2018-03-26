pub trait IntoStatic {
    type Result;
    fn into_static(self) -> Self::Result;
}

impl<Item: IntoStatic> IntoStatic for Vec<Item>
{
    type Result = Vec<Item::Result>;
    fn into_static(self) -> Self::Result {
        self.into_iter()
            .map(|item| item.into_static())
            .collect()
    }
}

impl<Item: IntoStatic> IntoStatic for Option<Item> {
    type Result = Option<Item::Result>;
    fn into_static(self) -> Self::Result {
        self.map(Item::into_static)
    }
}

impl<A: IntoStatic, B: IntoStatic> IntoStatic for (A, B) {
    type Result = (A::Result, B::Result);
    fn into_static(self) -> Self::Result {
        let (a, b) = self;
        (
            a.into_static(),
            b.into_static(),
        )
    }
}
