//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::column_common::{ColumnMode, Config, TableRow};
use comfy_table::{CellAlignment, ColumnConstraint, ContentArrangement, Table};
use libc::{nl_langinfo, CODESET, EXIT_FAILURE};
use serde_json::json;
use std::{
    collections::HashMap,
    ffi::CStr,
    io::{self, stdout, BufRead, BufReader, Read, Write},
};
use termion::terminal_size;
use uucore::error::{UResult, USimpleError};

/// TABCHAR_CELLS
const TABCHAR_CELLS: usize = 8;
/// Separator index
const SEPARATOR_INDEX: usize = 8;

/// Read input function
pub fn read_input<R: Read>(reader: R, config: &mut Config) -> io::Result<()> {
    let buf_reader = BufReader::new(reader);
    let mut separator = " ".to_owned();
    if let Some(ref sep) = config.input_separator {
        separator = sep.clone();
    }

    for byte_line in buf_reader.split(b'\n') {
        let byte_line = byte_line?;
        let mut line = String::new();

        for &byte in &byte_line {
            if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                line.push(byte as char);
            } else {
                line.push_str(&format!("\\x{:02x}", byte));
            }
        }

        let split_line: Vec<String>;
        match config.mode {
            ColumnMode::Table => {
                if separator == " ".to_owned() {
                    split_line = line
                        .split_terminator(&separator)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                } else {
                    split_line = line.split(&separator).map(|s| s.to_string()).collect();
                }
            }
            _ => {
                split_line = vec![line];
            }
        }

        for cell in split_line.iter() {
            if cell.len() > config.maxlength {
                config.maxlength = cell.len();
            }
        }

        // keep empty lines
        if !config.keep_empty_lines && split_line.is_empty() {
            continue;
        }
        config.ents.push(split_line);
    }
    Ok(())
}

/// Table main function
pub fn table_main(config: &mut Config) -> UResult<()> {
    // table columns limit
    if let Some(merge_index) = config.table_columns_limit {
        if merge_index < 1 {
            return Err(USimpleError::new(
                EXIT_FAILURE,
                "Error: invalid column limit",
            ));
        }
        let merge_index = merge_index - 1;
        for row in config.ents.iter_mut() {
            let mut new_cells: Vec<String> = Vec::new();
            if row.len() > merge_index {
                let merged_content = row[merge_index..].join(" ");
                new_cells.extend(row.iter().take(merge_index).map(|s| s.to_string()));
                new_cells.push(merged_content);
                *row = new_cells;
            }
        }
    }

    // Tree main
    if config.tree.is_some() {
        tree_main(config)?;
    }

    // Table order
    if config.table_order.is_some() {
        reorder_columns(config)?;
    }

    // Table truncate the col
    if !config.table_truncate.is_none() {
        truncate_columns(config)?;
    }

    // Table wrap the col
    if !config.table_wrap.is_none() {
        wrap_columns(config)?;
    }

    // JSON
    if config.json {
        print_json(config);
        return Ok(());
    }

    // Create the table
    let mut table = Table::new();

    // Assign value to table
    for cells in config.ents.iter() {
        table.add_row(cells);
    }

    // Set outer border
    let mut preset = "                   ".to_string();
    if let Some(separator) = &config.output_separator {
        if SEPARATOR_INDEX + 1 < preset.len() {
            preset.replace_range(SEPARATOR_INDEX..SEPARATOR_INDEX + 1, separator);
        }
    }

    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(preset.as_str());

    // Table col right
    if let Some(right_indices) = &config.table_right {
        // println!("right_indices = {:?}", right_indices);
        for (column_index, column) in table.column_iter_mut().enumerate() {
            if right_indices.contains(&column_index) {
                column.set_cell_alignment(CellAlignment::Right);
            }
        }
    }
    let len = table.column_iter().count() - 1;

    // Set the padding
    for (column_index, column) in table.column_iter_mut().enumerate() {
        if config.output_separator.is_some() {
            column.set_padding((0, 0));
        } else {
            if column_index == len {
                column.set_padding((0, 0));
            } else {
                column.set_padding((0, 2));
            }
        }
    }

    // Set the table header
    if let Some(table_columns) = &config.table_columns {
        if !config.table_noheadings {
            table.set_header(table_columns.clone());
        }
    }

    // table_hide
    if let Some(hide_indices) = &config.table_hide {
        for &index in hide_indices {
            table
                .column_mut(index)
                .unwrap()
                .set_constraint(ColumnConstraint::Hidden);
        }
    }

    if table.lines().count() == 1 {
        for row in table.lines() {
            println!("{}", row.replace(" ", ""));
        }
    } else if config.table_wrap.is_some() {
        println!("{}", table.trim_fmt());
    } else {
        println!("{}", trim_fmt_(config, table));
    }

    Ok(())
}

/// Fillcols main function
pub fn fillcols_main(config: &mut Config) {
    let termwidth = config.termwidth.unwrap();
    config.maxlength = (config.maxlength + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1);
    let numcols = termwidth / config.maxlength.max(1);
    let numcols = if numcols == 0 { 1 } else { numcols };
    let numrows = config.ents.len() / numcols
        + if config.ents.len() % numcols != 0 {
            1
        } else {
            0
        };

    for row in 0..numrows {
        let mut endcol = config.maxlength;
        let mut base = row;
        let mut chcnt = 0;

        for _col in 0..numcols {
            if let Some(ent) = config.ents.get(base) {
                for item in ent {
                    print!("{}", item);
                    stdout().flush().unwrap();
                    chcnt += config.width(item);
                    base += numrows;
                    if base >= config.ents.len() {
                        break;
                    }
                    loop {
                        let cnt = (chcnt + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1);
                        if cnt <= endcol {
                            print!("\t");
                            stdout().flush().unwrap();
                            chcnt = cnt;
                        } else {
                            break;
                        }
                    }
                    endcol += config.maxlength;
                }
            }
        }
        println!();
    }
}

/// Fillrows main function
pub fn fillrows_main(config: &mut Config) {
    config.maxlength = (config.maxlength + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1);
    let numcols = config.termwidth.unwrap() / config.maxlength;
    let mut endcol = config.maxlength;
    let mut chcnt = 0;
    let mut col = 0;
    let mut nents = config.ents.len();

    for lp in config.ents.iter() {
        for item in lp {
            print!("{}", item);
            stdout().flush().unwrap();
            chcnt += config.width(item);
        }

        nents -= 1;
        if nents == 0 {
            break;
        }

        col += 1;
        if col == numcols {
            col = 0;
            chcnt = 0;
            endcol = config.maxlength;
            println!();
        } else {
            loop {
                let cnt = (chcnt + TABCHAR_CELLS) & !(TABCHAR_CELLS - 1);
                if cnt <= endcol {
                    print!("\t");
                    stdout().flush().unwrap();
                    chcnt = cnt;
                } else {
                    break;
                }
            }
            endcol += config.maxlength;
        }
    }

    if chcnt > 0 {
        println!();
    }
}

/// Show main function
pub fn simple_main(config: &Config) {
    for row in &config.ents {
        for (i, cell) in row.iter().enumerate() {
            for byte in cell.bytes() {
                if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                    print!("{}", byte as char);
                } else {
                    print!("\\x{:02x}", byte);
                }
            }
            if i < row.len() - 1 {
                print!(" ");
            }
        }
        println!();
    }
}

/// Get last order index
pub fn get_last_order_index(table_columns: Vec<String>, table_order: Vec<String>) -> usize {
    let index = table_columns
        .iter()
        .position(|res| res == &table_order[table_order.len() - 1])
        .unwrap();
    index
}

/// Trim fmt
pub fn trim_fmt_(config: &mut Config, mut table: Table) -> String {
    let mut column_count = 0;
    for col in table.column_iter() {
        if !col.is_hidden() {
            column_count += 1;
        }
    }

    let mut last_column_index: usize = 0;
    let mut table_columns: Vec<String> = Vec::new();
    let mut table_order: Vec<String> = Vec::new();
    let mut table_hide: Vec<usize> = Vec::new();

    if let Some(result) = &config.table_columns {
        table_columns = result.to_owned();
    }
    if let Some(result) = &config.table_order {
        table_order = result.to_owned();
    }
    if let Some(result) = &config.table_hide {
        table_hide = result.to_owned();
    }

    if config.table_columns.is_some() && config.table_order.is_some() {
        last_column_index = get_last_order_index(table_columns, table_order);
    } else if config.table_columns.is_some() && config.table_hide.is_some() {
        for (index, _column) in table_columns.iter().enumerate() {
            if !table_hide.contains(&index) {
                last_column_index = index;
            }
        }
    } else {
        last_column_index = table.column_count() - 1;
    }

    let mut last_column_vec: Vec<String> = Vec::new();

    if column_count > 0 {
        if config.table_columns.is_some() && !config.table_noheadings {
            let cell_iter = table.column_cells_with_header_iter(last_column_index);
            for cell in cell_iter {
                match cell {
                    Some(cell) => last_column_vec.push(cell.content().to_string()),
                    None => last_column_vec.push("".to_string()),
                }
            }
        } else {
            let cell_iter = table.column_cells_iter(last_column_index);
            for cell in cell_iter {
                match cell {
                    Some(cell) => last_column_vec.push(cell.content().to_string()),
                    None => last_column_vec.push("".to_string()),
                }
            }
        }
    }

    let mut table_format_vec = Vec::new();
    for row in table.lines() {
        table_format_vec.push(row);
    }

    let mut start_index = 0;
    for (i, last_col_str) in last_column_vec.iter().enumerate() {
        if last_col_str.is_empty() {
            continue;
        } else {
            let row_str = &table_format_vec[i];
            if let Some(pos) = row_str.rfind(last_col_str) {
                start_index = pos;
                break;
            }
        }
    }

    let mut table_format_vec: Vec<String> = Vec::new();

    for row in table.lines() {
        table_format_vec.push(row.to_string());
    }

    for i in 0..table_format_vec.len() {
        if table_format_vec[i].trim().is_empty() && last_column_vec[i].is_empty() {
            if start_index <= table_format_vec[i].len() {
                table_format_vec[i].truncate(start_index);
            }
        } else if !last_column_vec[i].is_empty() {
            if let Some(pos) = table_format_vec[i].rfind(&last_column_vec[i]) {
                table_format_vec[i].truncate(pos + last_column_vec[i].len());
            }
        }
    }

    table_format_vec.join("\n")
}

/// Reorder columns
pub fn reorder_columns(config: &mut Config) -> UResult<()> {
    let table_columns = if let Some(table_columns) = &config.table_columns {
        table_columns.clone()
    } else {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Error: No table columns specified.",
        ));
    };

    let mut order_indices: Vec<usize> = Vec::new();
    if let Some(table_order) = &config.table_order {
        for order in table_order {
            if let Some(index) = table_columns.iter().position(|column| column == order) {
                order_indices.push(index);
            } else {
                return Err(USimpleError::new(
                    EXIT_FAILURE,
                    "Error: order contains invalid column name",
                ));
            }
        }
    }

    let mut non_order_indices: Vec<usize> = Vec::new();
    for (index, _) in table_columns.iter().enumerate() {
        if !order_indices.contains(&index) {
            non_order_indices.push(index);
        }
    }

    let mut final_order_indices: Vec<usize> = vec![0; table_columns.len()];
    let mut order_iter = order_indices.iter().peekable();
    let mut non_order_iter = non_order_indices.iter().peekable();

    for (index, _) in table_columns.iter().enumerate() {
        if non_order_indices.contains(&index) {
            final_order_indices[index] = *non_order_iter.next().unwrap();
        } else if order_indices.contains(&index) {
            final_order_indices[index] = *order_iter.next().unwrap();
        }
    }

    config.ents = config
        .ents
        .iter()
        .map(|row| {
            final_order_indices
                .iter()
                .map(|&i| row[i].clone())
                .collect()
        })
        .collect();

    config.table_columns = Some(
        final_order_indices
            .iter()
            .map(|&i| table_columns[i].clone())
            .collect(),
    );

    Ok(())
}

/// Calculate column lengths
pub fn calculate_column_lengths(config: &Config) -> (Vec<usize>, Vec<usize>, usize, Vec<usize>) {
    let num_columns = config.ents[0].len();

    let table_hide_index = if let Some(table_hide) = &config.table_hide {
        table_hide.clone()
    } else {
        Vec::new()
    };

    let mut col_cells_and_titles_max_lengths: Vec<usize> = vec![0; num_columns];
    let mut title_columns_lengths: Vec<usize> = vec![0; num_columns];

    for row in &config.ents {
        for (i, item) in row.iter().enumerate() {
            if !table_hide_index.contains(&i) {
                col_cells_and_titles_max_lengths[i] =
                    col_cells_and_titles_max_lengths[i].max(item.len());
            }
        }
    }

    if let Some(table_columns) = &config.table_columns {
        for (i, column) in table_columns.iter().enumerate() {
            if !table_hide_index.contains(&i) {
                title_columns_lengths[i] = column.len();
                col_cells_and_titles_max_lengths[i] =
                    col_cells_and_titles_max_lengths[i].max(column.len());
            }
        }
    }

    for &index in &table_hide_index {
        col_cells_and_titles_max_lengths[index] = 0;
        title_columns_lengths[index] = 0;
    }

    let termin_width = config.termwidth.unwrap();

    (
        col_cells_and_titles_max_lengths,
        title_columns_lengths,
        termin_width,
        table_hide_index,
    )
}

/// Truncate columns
pub fn truncate_columns(config: &mut Config) -> UResult<()> {
    // Implementation for truncating columns
    if config.ents.is_empty() {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Error: Empty table. Cannot truncate columns.",
        ));
    }

    let num_columns = config.ents[0].len();

    let (col_cells_and_titles_max_lengths, title_columns_lengths, termin_width, table_hide_index) =
        calculate_column_lengths(config);

    let mut temp_lengths: usize = 0;
    let mut table_truncate_index: Vec<usize> = Vec::new();

    if let Some(table_truncate) = &config.table_truncate {
        table_truncate_index.extend(table_truncate);
        table_truncate_index.sort();

        for (index, &len) in col_cells_and_titles_max_lengths.iter().enumerate() {
            if !table_truncate_index.contains(&index) && !table_hide_index.contains(&index) {
                temp_lengths += len;
            }
        }

        temp_lengths += 2 * (num_columns - 1 - table_hide_index.len());
    }

    if temp_lengths >= termin_width {
        for index in table_truncate_index {
            if index < num_columns {
                for row in &mut config.ents {
                    if let Some(content) = row.get_mut(index) {
                        *content = content.chars().take(title_columns_lengths[index]).collect();
                    }
                }
            }
        }
    } else {
        let truncate_length_sum = termin_width - temp_lengths;
        let average_length = truncate_length_sum / table_truncate_index.len();
        let remainder = truncate_length_sum % table_truncate_index.len();

        let mut truncate_lengths = vec![average_length; table_truncate_index.len()];
        for i in 0..remainder {
            truncate_lengths[i] += 1;
        }

        for (i, &index) in table_truncate_index.iter().enumerate() {
            if index < num_columns {
                for row in &mut config.ents {
                    if let Some(content) = row.get_mut(index) {
                        *content = content.chars().take(truncate_lengths[i]).collect();
                    }
                }
            }
        }
    }

    Ok(())
}

/// Wrap columns
pub fn wrap_columns(config: &mut Config) -> UResult<()> {
    if config.ents.is_empty() {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Error: Empty table. Cannot insert newlines in columns.",
        ));
    }

    let num_columns = config.ents[0].len();

    let (col_cells_and_titles_max_lengths, title_columns_lengths, termin_width, table_hide_index) =
        calculate_column_lengths(config);

    let mut temp_lengths: usize = 0;
    let mut table_wrap_index: Vec<usize> = Vec::new();

    if let Some(table_wrap) = &config.table_wrap {
        table_wrap_index.extend(table_wrap);
        table_wrap_index.sort();

        for (index, &len) in col_cells_and_titles_max_lengths.iter().enumerate() {
            if !table_wrap_index.contains(&index) && !table_hide_index.contains(&index) {
                temp_lengths += len;
            }
        }

        temp_lengths += 2 * (num_columns - 1 - table_hide_index.len());
    }

    if temp_lengths >= termin_width {
        for index in table_wrap_index {
            if index < num_columns {
                for row in &mut config.ents {
                    if let Some(content) = row.get_mut(index) {
                        let wrapped_content: String = content
                            .chars()
                            .collect::<Vec<_>>()
                            .chunks(title_columns_lengths[index])
                            .map(|chunk| chunk.iter().collect::<String>())
                            .collect::<Vec<_>>()
                            .join("\n");
                        *content = wrapped_content;
                    }
                }
            }
        }
    } else {
        let wrap_length_sum = termin_width - temp_lengths;
        let average_length = wrap_length_sum / table_wrap_index.len();
        let remainder = wrap_length_sum % table_wrap_index.len();

        let mut wrap_lengths = vec![average_length; table_wrap_index.len()];
        for i in 0..remainder {
            wrap_lengths[i] += 1;
        }

        for (i, &index) in table_wrap_index.iter().enumerate() {
            if index < num_columns {
                for row in &mut config.ents {
                    if let Some(content) = row.get_mut(index) {
                        let wrapped_content: String = content
                            .chars()
                            .collect::<Vec<_>>()
                            .chunks(wrap_lengths[i])
                            .map(|chunk| chunk.iter().collect::<String>())
                            .collect::<Vec<_>>()
                            .join("\n");
                        *content = wrapped_content;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Print json
pub fn print_json(config: &mut Config) {
    let table_name = if let Some(table_name) = &config.table_name {
        table_name.clone()
    } else {
        "table".to_string()
    };

    let mut json_output = Vec::new();
    for row in &config.ents {
        let mut columns = HashMap::new();
        if let Some(table_columns) = &config.table_columns {
            for (i, column) in table_columns.iter().enumerate() {
                columns.insert(column.clone(), row[i].clone());
            }
        }
        json_output.push(TableRow { columns });
    }
    let json_obj = json!({ table_name: json_output });
    let json_output = serde_json::to_string_pretty(&json_obj).unwrap();
    println!("{}", json_output);
}

/// Print node tree
fn print_node_tree(
    nodes: &HashMap<i32, Vec<Vec<String>>>,
    parent_node: i32,
    depth: usize,
    parent_branches: &[bool],
    is_root: bool,
    output: &mut Vec<Vec<String>>,
    tree_index: usize,
) {
    let mut titlepadding_symbol = " ";
    let mut branch_symbol = "|-";
    let mut vertical_symbol = "| ";
    let mut right_symbol = "`-";

    if unsafe { CStr::from_ptr(nl_langinfo(CODESET)).to_str() } == Ok("UTF-8") {
        titlepadding_symbol = "  ";
        branch_symbol = "├─";
        vertical_symbol = "│ ";
        right_symbol = "└─";
    }

    if let Some(child_nodes) = nodes.get(&parent_node) {
        for (index, parts) in child_nodes.iter().enumerate() {
            let mut row = parts.clone();

            if !is_root {
                let mut branches_row = String::new();
                for &branch in parent_branches {
                    branches_row += if branch {
                        vertical_symbol
                    } else {
                        titlepadding_symbol
                    };
                }

                let is_last = index == child_nodes.len() - 1;
                branches_row += if is_last { right_symbol } else { branch_symbol };

                row[tree_index] = branches_row + &row[tree_index];
            }

            output.push(row);
            print_node_tree(
                nodes,
                parts[0].parse::<i32>().unwrap_or(0), // Ensure correct parsing
                depth + 1,
                &[parent_branches, &[index != child_nodes.len() - 1]].concat(),
                false,
                output,
                tree_index,
            );
        }
    }
}

/// Tree main function
pub fn tree_main(config: &mut Config) -> UResult<()> {
    // get the value of tree_id and tree_parent
    let tree = config.tree.as_ref().unwrap();
    let tree_id = config.tree_id.as_ref().unwrap();
    let tree_parent = config.tree_parent.as_ref().unwrap();
    // if the value of tree tree_id and tree_parent is not available, return error
    if tree_id.is_empty() || tree_parent.is_empty() {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Error: tree-id and tree-parent must be specified",
        ));
    }

    // get the index of tree tree_id and tree_parent
    let tree_index = match parse_segment(tree, &config.table_columns) {
        Ok(column) => column,
        Err(e) => return Err(e),
    };
    let tree_parent_index = match parse_segment(tree_parent, &config.table_columns) {
        Ok(column) => column,
        Err(e) => return Err(e),
    };
    let mut nodes: HashMap<i32, Vec<Vec<String>>> = HashMap::new();
    for parts in &config.ents {
        let parent_node = parts[tree_parent_index].parse::<i32>().unwrap_or(0);
        nodes
            .entry(parent_node)
            .or_insert_with(Vec::new)
            .push(parts.clone());
    }
    let mut output: Vec<Vec<String>> = Vec::new();
    print_node_tree(&nodes, 0, 0, &[], true, &mut output, tree_index);
    // Ensure ents and output have the same length
    if config.ents.len() != output.len() {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Lengths of ents and output are different.",
        ));
    }
    config.ents = output;
    for row in &mut config.ents {
        if let Some(col) = row.get_mut(tree_index) {
            if col.starts_with("  ") && col.len() > 2 {
                *col = col[2..].to_string();
            } else if col.starts_with(" ") && col.len() > 1 {
                *col = col[1..].to_string();
            }
        }
    }
    Ok(())
}

/// Parse segment
pub fn parse_segment(segment: &str, table_columns: &Option<Vec<String>>) -> UResult<usize> {
    if let Ok(index) = segment.parse::<usize>() {
        Ok(index.saturating_sub(1))
    } else {
        if let Some(column_index) = table_columns
            .as_ref()
            .and_then(|columns| columns.iter().position(|name| name == segment))
        {
            Ok(column_index)
        } else {
            Err(USimpleError::new(
                EXIT_FAILURE,
                format!("undefined column name '{}'", segment),
            ))
        }
    }
}

/// Parse columns
pub fn parse_columns(
    option: Option<&String>,
    table_columns: &Option<Vec<String>>,
) -> UResult<Option<Vec<usize>>> {
    match option {
        Some(columns) => {
            let mut res = Vec::new();
            for column in columns
                .split(',')
                .flat_map(|segment| segment.split_whitespace())
            {
                match parse_segment(column, table_columns) {
                    Ok(s) => res.push(s),
                    Err(e) => return Err(e),
                }
            }
            Ok(Some(res))
        }
        None => Ok(None),
    }
}

/// Get terminal width
pub fn get_terminal_width(default_width: usize) -> usize {
    match terminal_size() {
        Ok((width, _)) => width as usize,
        Err(_) => default_width,
    }
}

/// Validate args
pub fn validate_args(config: &mut Config) -> UResult<()> {
    if config.tree.is_some() {
        config.mode = ColumnMode::Table;
        if config.tree_parent.is_none() || config.tree_id.is_none() {
            return Err(USimpleError::new(
                EXIT_FAILURE,
                "options --tree-id and --tree-parent are required for tree formatting",
            ));
        }
    }

    if config.mode != ColumnMode::Table {
        if config.table_order.is_some()
            || config.table_name.is_some()
            || config.table_wrap.is_some()
            || config.table_hide.is_some()
            || config.table_truncate.is_some()
            || config.table_right.is_some()
            || config.table_name.is_some()
        {
            return Err(USimpleError::new(
                EXIT_FAILURE,
                "Error: option --table required for all --table-*",
            ));
        }
    }

    if config.table_name.is_none() && config.json {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            "Error: option --table-columns required for --json",
        ));
    }

    Ok(())
}
