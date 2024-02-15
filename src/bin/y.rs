use y_sync::{
    awareness::Awareness,
    sync::{DefaultProtocol, Protocol},
};
use yrs::types::text::TextEvent;
use yrs::types::Delta;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::updates::encoder::EncoderV1;
use yrs::{
    Doc, GetString, Observable, ReadTxn, StateVector, Text, Transact, TransactionMut, Update,
};

fn observe_text_callback(txn: &TransactionMut, event: &TextEvent) {
    let deltas = event.delta(txn);
    for delta in deltas {
        match delta {
            Delta::Inserted(v, _) => {
                dbg!(v);
            }
            Delta::Deleted(n) => {
                dbg!(n);
            }
            Delta::Retain(n, _) => {
                dbg!(n);
            }
        }
    }
}

fn main() {
    manual_sync_example();
}

fn y_sync_example() {
    let doc = Doc::new();
    let text = doc.get_or_insert_text("article");
    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "hello");
    }

    let doc2 = Doc::new();
    let text2 = doc2.get_or_insert_text("article");
    {
        let mut txn = doc2.transact_mut();
        text2.insert(&mut txn, 0, " world");
    }

    let protocol = DefaultProtocol;
    let awareness = Awareness::new(doc);
    let mut encoder = EncoderV1::new();
    protocol.start(&awareness, &mut encoder).unwrap();

    todo!();
}

fn manual_sync_example() {
    let doc = Doc::new();
    let text = doc.get_or_insert_text("article");

    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 0, "hello");
        text.insert(&mut txn, 5, " world");
        // other rich text operations include formatting or inserting embedded elements
    } // transaction is automatically committed when dropped

    assert_eq!(text.get_string(&doc.transact()), "hello world".to_owned());

    // synchronize state with remote replica
    let remote_doc = Doc::new();
    let mut remote_text = remote_doc.get_or_insert_text("article");
    let remote_timestamp = remote_doc.transact().state_vector().encode_v1();

    // get update with contents not observed by remote_doc
    let update = doc
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&remote_timestamp).unwrap());
    // apply update on remote doc
    let _subscription = remote_text.observe(observe_text_callback);
    remote_doc
        .transact_mut()
        .apply_update(Update::decode_v1(&update).unwrap());

    {
        let mut txn = doc.transact_mut();
        text.insert(&mut txn, 3, "xxx");
    }
    let remote_timestamp = remote_doc.transact().state_vector().encode_v1();
    let update = doc
        .transact()
        .encode_diff_v1(&StateVector::decode_v1(&remote_timestamp).unwrap());
    remote_doc
        .transact_mut()
        .apply_update(Update::decode_v1(&update).unwrap());

    assert_eq!(
        text.get_string(&doc.transact()),
        remote_text.get_string(&remote_doc.transact())
    );
}
