use std::{collections::BTreeMap, mem::replace};

use crate::{
    comparisons::{Comparison, Comparisons},
    relations::{Position, Range, Relation, Relations},
    Versus,
};

#[derive(Debug)]
enum Comp<T> {
    Exact(T, T),
    Left(T),
    Right(T),
}
// impl<T> Comp<T> {
//     pub fn merge(&mut self,other:Self) {

//     }
// }

pub struct Comparator {
    pub intersection_left: bool,
}

impl Default for Comparator {
    fn default() -> Self {
        Self {
            intersection_left: Default::default(),
        }
    }
}

impl Comparator {
    pub fn set_intersection_left(self, intersection_left: bool) -> Self {
        Self { intersection_left }
    }

    pub fn compare(&self, left: Relations, right: Relations) -> Comparisons {
        let mut m: BTreeMap<Position, Comp<Relation>> = Default::default();
        for mut x in left {
            if x.search.is_none() {
                x.search = x.decl.typ.take();
            } else { x.decl.typ = None }
            m.insert(x.decl.clone(), Comp::Left(x));
        }
        for mut x in right {
            if x.search.is_none() {
                x.search = x.decl.typ.take();
            } else {x.decl.typ = None }
            match m.entry(x.decl.clone()) {
                std::collections::btree_map::Entry::Occupied(mut y) => {
                    let y = y.get_mut();
                    if let Comp::Left(yy) = y {
                        *y = Comp::Exact(
                            replace(
                                yy,
                                Relation {
                                    decl: Position {
                                        file: Default::default(),
                                        offset: Default::default(),
                                        len: Default::default(),
                                        typ: None,
                                    },
                                    duration: Default::default(),
                                    search: Default::default(),
                                    refs: Default::default(),
                                },
                            ),
                            x,
                        )
                    } else {
                        panic!();
                    }
                    // let y = y.get_mut();
                    // let left = if let Comp::Left(y) = y {
                    //     take(y)
                    // } else {
                    //     continue;
                    // };
                    // *y = Comp::Exact(left, x);
                }
                std::collections::btree_map::Entry::Vacant(y) => {
                    y.insert(Comp::Right(x));
                }
            }
        }

        let mut exact = vec![];
        let mut left = vec![];
        let mut right = vec![];

        for (k, v) in m {
            match v {
                Comp::Exact(mut left, mut right) => {
                    left.refs.sort();
                    right.refs.sort();
                    left.refs.dedup();
                    right.refs.dedup();
                    let mut intersection = vec![];
                    let mut per_file: BTreeMap<String, (Vec<Range>, Vec<Range>)> =
                        Default::default();
                    let mut remaining = left.refs;
                    let mut not_matched = vec![];
                    for r in right.refs {
                        if let Some(i) = remaining.iter().position(|x| x == &r) {
                            intersection.push(r);
                            remaining.swap_remove(i);
                        } else {
                            per_file
                                .entry(r.file.clone())
                                .or_insert((vec![], vec![]))
                                .1
                                .push(r.clone().into());
                            not_matched.push(r);
                        }
                    }
                    for l in &remaining {
                        per_file
                            .entry(l.file.clone())
                            .or_insert((vec![], vec![]))
                            .0
                            .push(l.clone().into());
                    }
                    let per_file = if self.intersection_left {
                        per_file
                            .into_iter()
                            .filter(|(_, (l, _))| !l.is_empty())
                            .map(|x| x.into())
                            .collect()
                    } else {
                        per_file.into_iter().map(|x| x.into()).collect()
                    };
                    let duration = Versus {
                        baseline: left.duration.unwrap_or_default(),
                        evaluated: right.duration.unwrap_or_default(),
                    }.into();
                    let search = Versus {
                        baseline: left.search.unwrap_or_default(),
                        evaluated: right.search.unwrap_or_default(),
                    }.into();
                    exact.push(Comparison {
                        decl: k,
                        duration,
                        search,
                        exact: intersection,
                        per_file,
                        left: remaining,
                        right: not_matched,
                        left_contained: vec![],
                        right_contained: vec![],
                    })
                }
                Comp::Left(l) => left.push(l),
                Comp::Right(r) => right.push(r),
            }
        }
        Comparisons {
            left_name: "".to_string(),
            right_name: "".to_string(),
            exact,
            left,
            right,
        }
    }
}


impl<T:Default> From<Versus<Option<T>>> for Option<Versus<T>> {
    fn from(x: Versus<Option<T>>) -> Self {
        if x.baseline.is_none() && x.evaluated.is_none() {
            None
        } else {
            Some(Versus {
                baseline: x.baseline.unwrap_or_default(),
                evaluated: x.evaluated.unwrap_or_default(),
            })
        }
    }
}