use swagger::{make_context, make_context_ty, new_context_type, Has, Pop, Push};

#[derive(Debug, Default, PartialEq, Eq)]
struct Item1(u32);
#[derive(Debug, Default, PartialEq, Eq)]
struct Item2;
#[derive(Debug, Default, PartialEq, Eq)]
struct Item3;

new_context_type!(ExtContext, ExtEmptyContext, Item1, Item2, Item3);

#[test]
fn context_macros_work_from_external_crate() {
    let ctx = ExtEmptyContext.push(Item3).push(Item2).push(Item1(42));

    let v: &Item1 = ctx.get();
    assert_eq!(v.0, 42);

    let (item2, ctx): (Item2, _) = ctx.pop();
    let (_item3, _ctx): (Item3, _) = ctx.pop();

    let _ = item2;
}

#[test]
fn make_context_macros_work_from_external_crate() {
    type Ctx = make_context_ty!(ExtContext, ExtEmptyContext, Item1, Item2, Item3);

    let ctx1: Ctx = make_context!(ExtContext, ExtEmptyContext, Item1(5), Item2, Item3);
    let ctx2: Ctx = ExtEmptyContext.push(Item3).push(Item2).push(Item1(5));

    assert_eq!(Has::<Item1>::get(&ctx1).0, 5);
    assert_eq!(ctx1, ctx2);
}
