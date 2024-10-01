use std::{
    collections::BTreeMap,
    env::args,
    fmt::{Display, Formatter, Result},
    fs::OpenOptions,
    io::{BufWriter, Write},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Coord(i32, i32);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Polyomino(Vec<Coord>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PolyominoSymmetryGroup {
    None,
    Mirror90,
    Mirror45,
    Rotation2Fold,
    Rotation2FoldMirror90,
    Rotation2FoldMirror45,
    Rotation4Fold,
    All,
}

impl Coord {
    fn rotate(self) -> Coord {
        let Coord(x, y) = self;
        Coord(-y, x)
    }

    fn transpose(self) -> Coord {
        let Coord(x, y) = self;
        Coord(y, x)
    }
}

impl Polyomino {
    fn rotate(&self) -> Polyomino {
        Polyomino(self.0.clone().into_iter().map(Coord::rotate).collect()).canonize_fixed()
    }

    fn transpose(&self) -> Polyomino {
        Polyomino(self.0.clone().into_iter().map(Coord::transpose).collect()).canonize_fixed()
    }

    fn canonize_fixed(&self) -> Polyomino {
        let min_x = self.0.iter().map(|coord| coord.0).min().unwrap_or(0);
        let min_y = self.0.iter().map(|coord| coord.1).min().unwrap_or(0);

        let mut normalized_coords: Vec<_> = self
            .0
            .iter()
            .map(|&Coord(x, y)| Coord(x - min_x, y - min_y))
            .collect();
        normalized_coords.sort();

        Polyomino(normalized_coords)
    }

    fn canonize_free(&self) -> (Polyomino, PolyominoSymmetryGroup) {
        let c0 = self.canonize_fixed();
        let c90 = self.rotate().canonize_fixed();
        let c180 = c90.rotate().canonize_fixed();
        let c270 = c180.rotate().canonize_fixed();
        let t0 = self.transpose().canonize_fixed();
        let t90 = t0.rotate().canonize_fixed();
        let t180 = t90.rotate().canonize_fixed();
        let t270 = t180.rotate().canonize_fixed();

        let symmetry_group = if c0 == c90 {
            if c0 == t0 {
                PolyominoSymmetryGroup::All
            } else {
                PolyominoSymmetryGroup::Rotation4Fold
            }
        } else if c0 == t0 || c0 == t180 {
            if c0 == c180 {
                PolyominoSymmetryGroup::Rotation2FoldMirror45
            } else {
                PolyominoSymmetryGroup::Mirror45
            }
        } else if c0 == t90 || c0 == t270 {
            if c0 == c180 {
                PolyominoSymmetryGroup::Rotation2FoldMirror90
            } else {
                PolyominoSymmetryGroup::Mirror90
            }
        } else if c0 == c180 {
            PolyominoSymmetryGroup::Rotation2Fold
        } else {
            PolyominoSymmetryGroup::None
        };

        let mut all = vec![c0, c90, c180, c270, t0, t90, t180, t270];
        all.sort();

        (all.remove(0), symmetry_group)
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl Display for Polyomino {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (i, coord) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            coord.fmt(f)?;
        }
        Ok(())
    }
}

fn main() {
    let up_to = args().collect::<Vec<_>>()[1].parse::<usize>().unwrap();

    let mut previous = vec![Polyomino(vec![Coord(0, 0)])];

    for n in 2..(up_to + 1) {
        let current = previous
            .iter()
            .flat_map(|polyomino| {
                polyomino.0.iter().flat_map(|&Coord(x, y)| {
                    vec![
                        v(polyomino, Coord(x + 1, y)),
                        v(polyomino, Coord(x - 1, y)),
                        v(polyomino, Coord(x, y + 1)),
                        v(polyomino, Coord(x, y - 1)),
                    ]
                    .into_iter()
                    .flat_map(|x| x)
                })
            })
            .collect::<BTreeMap<Polyomino, PolyominoSymmetryGroup>>();
        save(n, &current);
        previous = current
            .into_iter()
            .map(|(polyomino, _)| polyomino)
            .collect();
    }

    fn v(polyomino: &Polyomino, new_coord: Coord) -> Vec<(Polyomino, PolyominoSymmetryGroup)> {
        if polyomino.0.iter().any(|&coord| coord == new_coord) {
            vec![]
        } else {
            let mut result = polyomino.0.clone();
            result.push(new_coord);
            vec![Polyomino(result).canonize_free()]
        }
    }

    fn save(n: usize, current: &BTreeMap<Polyomino, PolyominoSymmetryGroup>) {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // Truncate file if it already exists
            .open(format!("{}.json", n))
            .unwrap();
        let mut writer = BufWriter::new(file);

        write!(writer, "{{").unwrap();
        for (i, (polyomino, symmetry_group)) in current.iter().enumerate() {
            if i == 0 {
                write!(writer, "\n\t\"{}\": \"{:?}\"", polyomino, symmetry_group).unwrap();
            } else {
                write!(writer, ",\n\t\"{}\": \"{:?}\"", polyomino, symmetry_group).unwrap();
            }
        }
        write!(writer, "\n}}\n").unwrap();
        writer.flush().unwrap();
    }
}
