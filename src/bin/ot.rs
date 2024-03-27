use operational_transform::{
    OperationSeq,
    Operation::{*},
};

fn main() {
    let s = "";

    let mut b = OperationSeq::from_iter([Retain(2), Insert("🥕".to_owned())]);

    let mut a = OperationSeq::default();
    a.retain(2);
    a.insert("👩‍❤️‍💋‍👩");

    let (a_prime, b_prime) = a.transform(&b).unwrap();
    dbg!(&a_prime);
    dbg!(&b_prime);

    let ab_prime = dbg!(a.compose(&b_prime).unwrap());
    let ba_prime = dbg!(b.compose(&a_prime).unwrap());

    //let after_ab_prime = ab_prime.apply(s).unwrap();
    //let after_ba_prime = ba_prime.apply(s).unwrap();

    assert_eq!(ab_prime, ba_prime);
    //assert_eq!(dbg!(after_ab_prime), after_ba_prime); 
}
