extern crate quickcheck;
extern crate rand;
extern crate std;

use std::mem;
use quickcheck::Arbitrary;
use quickcheck::Gen;

// This property ensures that all elements to the left of the node we're handed are less
// than the value that node contains, and all elements to the right greater than. Equal
// is not handled as duplicates are not allowed.
#[quickcheck]
fn ordering_property(bt: BinaryTree<i32, (i8, i8)>) -> bool {
    match bt {
        BinaryTree {metadata: _, value, left: Some(ref left), right: Some(ref right)} => {
            return left.iter().all(|t| value > t.value) && right.iter().all(|t| value < t.value)
        },
        BinaryTree {metadata: _, value, left: None, right: Some(ref right)} => {
            return right.iter().all(|t| value < t.value)
        },
        BinaryTree {metadata: _, value, left: Some(ref left), right: None} => {
            return left.iter().all(|t| value > t.value)
        },
        _ => true
    }
}

// This implementation of an AVL tree tracks height of left and right children as a
// 2-tuple of i8s that represent the height of the respective children.

// This property recursively walks the tree and verifies that the height metadata is
// in correspondence with the recursive calculation of height.
#[quickcheck]
fn height_is_maintained(bt: AvlTree<i32>) -> bool {
    match bt {
        BinaryTree {
            metadata: (ref left_height, ref right_height), value: _,
            left: Some(box BinaryTree {metadata: (ref left_left_height, ref left_right_height), ..}),
            right: Some(box BinaryTree {metadata: (ref right_left_height, ref right_right_height), ..})}
        => {
            *left_height == std::cmp::max(*left_left_height, *left_right_height) + 1 && *right_height == std::cmp::max(*right_left_height, *right_right_height) + 1
        },
        BinaryTree {
            metadata: (ref left_height, ref right_height), value: _,
            right: Some(box BinaryTree {metadata: (ref right_left_height, ref right_right_height), ..}),
            left: None}
        => {
            *right_height == std::cmp::max(*right_left_height, *right_right_height) + 1 && *left_height == 0
        },
        BinaryTree {
            metadata: (ref left_height, ref right_height), value: _,
            left: Some(box BinaryTree {metadata: (ref left_left_height, ref left_right_height), ..}),
            right: None}
        => {
            *left_height == std::cmp::max(*left_left_height, *left_right_height) + 1 && *right_height == 0
        },
        BinaryTree {metadata: (ref left_height, ref right_height), value: _, left: None, right: None} => {
            *left_height == 0 && *right_height == 0
        }
    }
}

// This property verifies that, for the node we're given, the height metadata never
// differs by more than 1. This is the definition of a tree being balanced.
#[quickcheck]
fn balance_property(bt: BinaryTree<i32, (i8, i8)>) -> bool {
    match bt {
        BinaryTree {metadata, ..} => ( metadata.0 - metadata.1 ) <= 1 && ( metadata.0 - metadata.1 ) >= -1
    }
}

#[derive(Debug,Clone)]
struct BinaryTree<V: Ord+Copy, M> {
        metadata: M,
        value: V,
        left: Option<Box<BinaryTree<V, M>>>,
        right: Option<Box<BinaryTree<V, M>>>
}

// An arbitrary tree is formed by starting with an empty tree and inserting an arbitrary
// number of arbitrary values. Generating the tree any other way would defeat the purpose
// of the property based tests.
impl Arbitrary for BinaryTree<i32, (i8, i8)> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let mut tree = BinaryTree {metadata: (0, 0), value: g.gen_range(-1000,1000), left: None, right: None};
        while g.gen() {
            tree.insert(g.gen_range(-1000,1000));
        }
        tree
    }
}

// The iterator stuff is only used in the quickcheck properties. Specifically for
// checking ordering.
impl <'a, V: Ord+Copy+Clone+Send, M: Copy+Clone+Send> BinaryTree<V, M> {
    #[allow(dead_code)]
    fn iter(&'a self) -> BinaryTreeIterator<'a, V, M> {
        BinaryTreeIterator {to_visit: vec![&self]}
    }
}

#[allow(dead_code)]
struct BinaryTreeIterator<'a, V: 'a+Ord+Copy+Clone+Send, M: 'a+Copy+Clone+Send> {
    to_visit: Vec<&'a BinaryTree<V, M>>
}

impl <'a, V: 'a+Ord+Copy+Clone+Send, M: 'a+Copy+Clone+Send> Iterator for BinaryTreeIterator<'a, V, M> {
    type Item = &'a BinaryTree<V, M>;

    // Do depth first search as an iterator.
    fn next(&mut self) -> Option<&'a BinaryTree<V, M>> {
        let ret = self.to_visit.pop();
        match ret {
            Some(&BinaryTree {left: Some(ref left), right: None, ..}) => {self.to_visit.push(left)},
            Some(&BinaryTree {left: None, right: Some(ref right), ..}) => {self.to_visit.push(right)},
            Some(&BinaryTree {left: Some(ref left), right: Some(ref right), ..}) => {
                self.to_visit.push(left);
                self.to_visit.push(right)
            },
            _ => ()
        }
        ret
    }
}

type AvlTree<'a, V: 'a> = BinaryTree<V, (i8, i8)>;

impl <'a> AvlTree<'a, i32> {
    #[allow(non_shorthand_field_patterns)]
    // As we recurse down, we build up an implicit insertion path on the stack.
    // If we do an insert succesfully (i.e.: it is not a duplicate value we are
    // attempting to insert), then we may or may not need to propagate up the
    // stack how much the heights changed. The return value tells the caller
    // how much the maximal height changed at our layer, so it can do the
    // appropriate logic to decide what bookkeeping changes it needs to do.
    fn insert(&mut self, new_value: i32) -> i8 {
        let ret = match *self {
            BinaryTree {value, left: None, right: None, ..} => {
                if new_value > value {
                    *self = BinaryTree {
                        metadata: (0, 1),
                        value: value,
                        right: Some(Box::new(BinaryTree {
                            metadata: (0, 0),
                            value: new_value,
                            left: None,
                            right: None
                        })),
                        left: None
                    }
                } else if new_value == value {
                   return 0 // we don't allow duplicates.
                } else {
                    *self = BinaryTree {
                        metadata: (1, 0),
                        value: value,
                        left: Some(Box::new(BinaryTree {
                            metadata: (0, 0),
                            value: new_value,
                            left: None,
                            right: None
                        })),
                        right: None
                    }
                }
                1
            }
            BinaryTree {metadata: (ref mut left_height, right_height), ref mut value, left: Some(ref mut left ), ..} if new_value < *value => {
                let incr = left.insert(new_value);
                *left_height += incr;
                assert!(incr < 2);
                if *left_height == right_height + 1 { incr } else { 0 }
            }
            BinaryTree {metadata: (ref mut left_height, right_height), ref mut value, ref mut left, ..} if new_value < *value => {
                assert_eq!(0, *left_height);

                *left = Some(Box::new(BinaryTree {value: new_value, metadata: (0, 0), left: None, right: None}));
                *left_height += 1;
                if *left_height == right_height + 1 { 1 } else { 0 }
            }
            BinaryTree {metadata: (left_height, ref mut right_height), ref mut value, right: Some(ref mut right), ..} if new_value > *value => {
                let incr = right.insert(new_value);
                *right_height += incr;
                assert!(incr < 2);
                if *right_height == left_height + 1 { incr } else { 0 }
            }
            BinaryTree {metadata: (left_height, ref mut right_height), ref mut value, right: ref mut right, ..} if new_value > *value => {
                assert_eq!(0, *right_height);

                *right = Some(Box::new(BinaryTree {value: new_value, metadata: (0, 0), left: None, right: None}));
                *right_height += 1;
                if *right_height == left_height + 1 { 1 } else { 0 }
            }
            BinaryTree {ref mut value, ..} if *value == new_value => {
                0 // this is a duplicate value, do nothing.
            }
            BinaryTree {..} => unreachable!()
        };
        self.balance();
        self.fix_metadata();
        ret
    }

    // For each child we have, set the metadata at our layer of the tree to be
    // 1 + max(left_height, right_height) where left_height and right_height are
    // the values stored in that child's metadata. This is more verbose than it ideally
    // would be because we have to match on every possible case.
    fn fix_metadata(&mut self) {
        match self {
            &mut BinaryTree {
                left: Some(box BinaryTree {metadata: (left_left, left_right), ..}),
                right: Some(box BinaryTree {metadata: (right_left, right_right), ..}),
                ..}
            => {
                self.metadata = (std::cmp::max(left_left, left_right) + 1, std::cmp::max(right_left, right_right) + 1);
            }
            &mut BinaryTree {left: None, right: Some(box BinaryTree {metadata: (right_left, right_right), ..}), ..}
            => {
                self.metadata = (0, std::cmp::max(right_left, right_right) + 1);
            }
            &mut BinaryTree {left: Some(box BinaryTree {metadata: (left_left, left_right), ..}), right: None, ..}
            => {
                self.metadata = (std::cmp::max(left_left, left_right) + 1, 0);
            }
            &mut BinaryTree {left: None, right: None, ..}
            => {
                self.metadata = (0, 0);
            }
        }
    }

    // Rotations aren't inherently that complicated, but they sure are in Rust!
    // (In other words, you're on your own here for now.)
    fn rotate_left(&mut self) {
        let mut right: &mut Option<Box<AvlTree<i32>>> = &mut Some(Box::new(BinaryTree {metadata: (0,0), value: 0, right: None, left: None}));
        mem::swap(right, &mut self.right);
        mem::swap(&mut self.right, &mut right.as_mut().unwrap().left);
        mem::swap(&mut **right.as_mut().unwrap().left.as_mut().unwrap(), self);
        self.fix_metadata();
        right.as_mut().unwrap().left.as_mut().unwrap().fix_metadata();
        right.as_mut().unwrap().fix_metadata();
        mem::swap(self, right.as_mut().unwrap());
    }

    fn rotate_right(&mut self) {
        let mut left: &mut Option<Box<AvlTree<i32>>> = &mut Some(Box::new(BinaryTree {metadata: (0,0), value: 0, left: None, right: None}));
        mem::swap(left, &mut self.left);
        mem::swap(&mut self.left, &mut left.as_mut().unwrap().right);
        mem::swap(&mut **left.as_mut().unwrap().right.as_mut().unwrap(), self);
        self.fix_metadata();
        left.as_mut().unwrap().right.as_mut().unwrap().fix_metadata();
        left.as_mut().unwrap().fix_metadata();
        mem::swap(self, left.as_mut().unwrap());
    }

    // As stated above, the definition of a balanced tree is one where the height
    // of any pair of children does not differ by more than one. As a result, to
    // ensure that our tree is balanced, as soon as difference hits 2 or -2, we
    // do the appropriate rotations. What the appropriate rotations are is a bit
    // subtle, and out of scope for explaining here. However, there's a pretty good
    // explanation at the link below this comment, and on wikipedia.

    // http://www.cise.ufl.edu/~nemo/cop3530/AVL-Tree-Rotations.pdf
    fn balance(&mut self) {
        let difference: i8 = self.metadata.0 - self.metadata.1;

        // if this fails, all hope is lost.
        assert!(difference <= 2);
        assert!(difference >= -2);

        if difference == 2  {
            match self.left {
                Some(ref mut left @ box BinaryTree {left: None, right: Some(_), ..}) => {
                        left.rotate_left();
                }
                Some(ref mut left @ box BinaryTree {left: Some(_), right: Some(_), ..}) if left.metadata.0 - left.metadata.1 < 0 => {
                        left.rotate_left();
                }
                _ => ()
            }
            self.rotate_right();
        } else if difference == -2 {
            match self.right {
                Some(ref mut right @ box BinaryTree {right: None, left: Some(_), ..}) => {
                        right.rotate_right();
                }
                Some(ref mut right @ box BinaryTree {right: Some(_), left: Some(_), ..}) if right.metadata.0 - right.metadata.1 > 0 => {
                        right.rotate_right();
                }
                _ => ()
            }
            self.rotate_left();
        } else if difference < -2 && difference > 2 {
            unreachable!()
        }
    }
}
