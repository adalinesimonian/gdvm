// SPDX-FileCopyrightText: Copyright (C) 2026 Adaline Simonian
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of gdvm.
//
// gdvm is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

use toml_edit::DocumentMut;

pub(super) fn document_get<'a>(
    document: &'a DocumentMut,
    path: &str,
) -> Option<&'a toml_edit::Item> {
    let mut segments = path.split('.');
    let mut item = document.get(segments.next()?)?;

    for segment in segments {
        item = item.get(segment)?;
    }

    Some(item)
}

pub(super) fn document_set(document: &mut DocumentMut, path: &str, item: toml_edit::Item) {
    let mut segments: Vec<&str> = path.split('.').collect();
    let Some(leaf) = segments.pop() else {
        return;
    };

    let mut table = document.as_table_mut();
    for segment in segments {
        let is_new = !table.contains_key(segment);
        let entry = table[segment].or_insert(toml_edit::table());
        let Some(next) = entry.as_table_mut() else {
            // Leave whatever was found alone so as not to write over anything
            // the user put there.
            return;
        };

        if is_new {
            next.set_dotted(true);
        }

        table = next;
    }
    table[leaf] = item;
}

pub(super) fn document_remove(document: &mut DocumentMut, path: &str) {
    let segments: Vec<&str> = path.split('.').collect();
    let Some((leaf, parents)) = segments.split_last() else {
        return;
    };
    let mut table = document.as_table_mut();
    let mut visited: Vec<&str> = Vec::new();

    for segment in parents {
        let Some(next) = table.get_mut(segment).and_then(|item| item.as_table_mut()) else {
            return;
        };

        visited.push(segment);
        table = next;
    }

    table.remove(leaf);

    while let Some(segment) = visited.pop() {
        let mut table = document.as_table_mut();

        for parent in &visited {
            let Some(next) = table.get_mut(parent).and_then(|item| item.as_table_mut()) else {
                return;
            };

            table = next;
        }

        let is_empty = table
            .get(segment)
            .and_then(|item| item.as_table())
            .is_some_and(|child| child.is_empty());

        if !is_empty {
            return;
        }

        table.remove(segment);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(contents: &str) -> DocumentMut {
        contents.parse().unwrap()
    }

    #[test]
    fn test_get_walks_dotted_paths() {
        let doc = document("[prune]\nmax-age-days = 7\n");

        assert!(document_get(&doc, "prune.max-age-days").is_some());
        assert!(document_get(&doc, "prune.other").is_none());
        assert!(document_get(&doc, "missing.key").is_none());
    }

    #[test]
    fn test_set_creates_missing_tables_and_keeps_the_rest() {
        let mut doc = document("# hello\nkeep = true\n");

        document_set(&mut doc, "prune.max-age-days", toml_edit::value(7));

        assert!(doc.to_string().contains("# hello"));
        assert!(doc.to_string().contains("keep = true"));
        assert!(doc.to_string().contains("prune.max-age-days = 7"));
        assert_eq!(
            document_get(&doc, "prune.max-age-days").and_then(|item| item.as_integer()),
            Some(7)
        );
    }

    #[test]
    fn test_set_keeps_the_style_of_tables_the_user_wrote() {
        let mut doc = document("[prune]\nmine = 1\n");

        document_set(&mut doc, "prune.max-age-days", toml_edit::value(7));

        assert!(doc.to_string().contains("[prune]"));
        assert!(doc.to_string().contains("max-age-days = 7"));
        assert!(!doc.to_string().contains("prune.max-age-days"));
    }

    #[test]
    fn test_remove_prunes_tables_it_empties() {
        let mut doc = document("keep = true\n\n[prune]\nmax-age-days = 7\n");

        document_remove(&mut doc, "prune.max-age-days");

        assert!(!doc.to_string().contains("prune"));
        assert!(doc.to_string().contains("keep = true"));
    }

    #[test]
    fn test_remove_keeps_tables_the_user_still_uses() {
        let mut doc = document("[prune]\nmax-age-days = 7\nmine = 1\n");

        document_remove(&mut doc, "prune.max-age-days");

        assert!(doc.to_string().contains("mine = 1"));
        assert!(document_get(&doc, "prune.max-age-days").is_none());
    }
}
