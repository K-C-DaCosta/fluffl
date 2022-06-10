pub struct GroupIterator<Iter, Key, KeyCB, ItemCB>
where
    Iter: Iterator,
    Key: PartialEq,
    ItemCB: Fn(&Iter::Item) + 'static,
    KeyCB: Fn(&Iter::Item) -> Key + 'static,
{
    prev_val: Option<Iter::Item>,
    prev_key: Option<Key>,
    item_stream: Iter,
    per_item: ItemCB,
    first_key: Option<Key>,
    get_key: KeyCB,
}
impl<Iter, Key, KeyCB, ItemCB> GroupIterator<Iter, Key, KeyCB, ItemCB>
where
    Iter: Iterator,
    Key: PartialEq,
    ItemCB: Fn(&Iter::Item),
    KeyCB: Fn(&Iter::Item) -> Key,
{
    pub fn new(iterator: Iter, key_cb: KeyCB, item_cb: ItemCB) -> Self {
        Self {
            prev_key: None,
            prev_val: None,
            item_stream: iterator,
            get_key: key_cb,
            per_item: item_cb,
            first_key: None,
        }
    }
}

impl<Iter, Key, KeyCB, ItemCB> Iterator for GroupIterator<Iter, Key, KeyCB, ItemCB>
where
    Iter: Iterator,
    Key: PartialEq + Copy,
    ItemCB: Fn(&Iter::Item) + Copy + 'static,
    KeyCB: Fn(&Iter::Item) -> Key + Copy + 'static,
    Iter::Item: Copy,
{
    type Item = Option<Iter::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.item_stream.next();
        let get_key = self.get_key;
        let per_item = self.per_item;

        match (item, self.prev_key.as_ref()) {
            (Some(i), None) => {
                let key = get_key(&i);
                self.first_key = Some(key);
                self.prev_val = Some(i);
                self.prev_key = Some(key);
                per_item(&i);
                Some(None)
            }
            (Some(i), Some(prev_key)) => {
                let prev_val = self.prev_val;
                let current_key = get_key(&i);
                let key = get_key(&i);
                
                
                
                

                let group = if current_key.eq(prev_key) == false {
                    Some(prev_val)
                } else {
                    Some(None)
                };

                self.prev_val = Some(i);
                self.prev_key = Some(key);
                per_item(&i);

                group
            }
            _ => self.prev_val.take().map(|a| Some(a)),
        }
    }
}

#[test]
fn sanity() {
    let list:Vec<u32> = vec![
        0, 0,0,
    ];

    GroupIterator::new(
        list.iter().enumerate(),
        |&a| a.1,
        |b| println!("item:{:?}", b),
    )
    .filter_map(|a| a)
    .for_each(|b| println!("group is {}", b.1));
}
