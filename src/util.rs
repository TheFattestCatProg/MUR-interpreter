use std::collections::LinkedList;


pub fn as_linked<T>(value: T) -> LinkedList<T> {
    let mut list = LinkedList::new();
    list.push_back(value);

    list
}