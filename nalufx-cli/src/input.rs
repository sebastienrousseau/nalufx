//! Reading returns and series: from CSV files, the command line or live
//! prices.

use crate::cli::ReturnsSource;
use crate::error::CliError;
use ndarray::Array2;
use std::io::Read;
use std::path::Path;

/// Columns that label a row rather than hold a return, skipped when first.
const LABEL_COLUMNS: [&str; 3] = ["date", "day", "time"];

/// Asset names and their returns, one row per day.
pub type Returns = (Vec<String>, Array2<f64>);

/// The returns the source names: read from its file, or fetched live.
///
/// # Errors
///
/// [`CliError::Input`] for an unreadable file; [`CliError::Provider`] if
/// the price feed fails.
pub async fn returns(source: &ReturnsSource) -> Result<Returns, CliError> {
    match &source.file {
        Some(path) => read_returns(path),
        None => fetch(&source.assets).await,
    }
}

/// Daily returns for each ticker from the live price feed.
async fn fetch(tickers: &[String]) -> Result<Returns, CliError> {
    let tickers: Vec<String> =
        tickers.iter().map(|t| t.trim().to_uppercase()).filter(|t| !t.is_empty()).collect();
    nalufx::services::fetch_data_svc::fetch_returns(&tickers).await.map_err(|e| {
        CliError::Provider {
            message: format!("could not fetch prices: {e}"),
            hint: Some(
                "check the tickers and your connection, or use --file with a CSV of returns".into(),
            ),
        }
    })
}

/// A CSV of returns: a header of asset names, then one row per day. A
/// first column named `date`, `day` or `time` is skipped.
///
/// # Errors
///
/// [`CliError::Input`] naming the file, row and column of the first
/// problem.
pub fn read_returns(path: &Path) -> Result<Returns, CliError> {
    returns_from(open(path)?, &path.display().to_string())
}

/// Returns from CSV text in any reader; `source` names it in errors.
///
/// # Errors
///
/// As [`read_returns`].
pub fn returns_from(reader: impl Read, source: &str) -> Result<Returns, CliError> {
    let (header, rows) = parse_csv(reader, source)?;
    let skip = usize::from(
        header.first().is_some_and(|h| LABEL_COLUMNS.contains(&h.to_lowercase().as_str())),
    );
    let assets: Vec<String> = header[skip..].to_vec();
    if assets.is_empty() || rows.is_empty() {
        return Err(CliError::input(
            format!("{source}: no assets or no rows"),
            "the first line names the assets; each line after it is one day",
        ));
    }
    let mut values = Vec::with_capacity(rows.len() * assets.len());
    for (r, row) in rows.iter().enumerate() {
        values.extend(parse_row(source, &header, row, r + 2, skip)?);
    }
    let matrix = Array2::from_shape_vec((rows.len(), assets.len()), values)
        .map_err(|e| CliError::input(format!("{source}: {e}"), "check the file's shape"))?;
    Ok((assets, matrix))
}

/// One row's numbers, skipping the first `skip` cells; `line` is its line
/// in the file, for errors.
fn parse_row(
    source: &str,
    header: &[String],
    row: &[String],
    line: usize,
    skip: usize,
) -> Result<Vec<f64>, CliError> {
    if row.len() != header.len() {
        return Err(CliError::input(
            format!("{source}: row {line} has {} values, the header {}", row.len(), header.len()),
            "every row needs one value per column",
        ));
    }
    row.iter()
        .enumerate()
        .skip(skip)
        .map(|(c, cell)| number(source, line, &header[c], cell))
        .collect()
}

/// One column of a CSV, by name, or the last column.
///
/// # Errors
///
/// [`CliError::Input`] if the column is missing or a value is not a number.
pub fn read_series(path: &Path, column: Option<&str>) -> Result<Vec<f64>, CliError> {
    let source = path.display().to_string();
    let (header, rows) = parse_csv(open(path)?, &source)?;
    let index = match column {
        Some(name) => header.iter().position(|h| h == name).ok_or_else(|| {
            CliError::input(
                format!("{source}: no column `{name}`"),
                format!("the columns are: {}", header.join(", ")),
            )
        })?,
        None => header.len().saturating_sub(1),
    };
    rows.iter()
        .enumerate()
        .map(|(r, row)| {
            let cell = row.get(index).map_or("", String::as_str);
            number(&source, r + 2, &header[index], cell)
        })
        .collect()
}

/// A file's text, or standard input's for `-`.
///
/// # Errors
///
/// [`CliError::Input`] if it cannot be read.
pub fn read_text(path: &Path) -> Result<String, CliError> {
    let mut text = String::new();
    let read = if path == Path::new("-") {
        std::io::stdin().read_to_string(&mut text).map(|_| ())
    } else {
        std::fs::read_to_string(path).map(|t| text = t)
    };
    read.map_err(|e| {
        CliError::input(
            format!("{}: {e}", path.display()),
            "check the path, or pass - to read standard input",
        )
    })?;
    Ok(text)
}

/// A file, opened for reading.
fn open(path: &Path) -> Result<std::fs::File, CliError> {
    std::fs::File::open(path).map_err(|e| {
        CliError::input(
            format!("{}: {e}", path.display()),
            "check the path; nalufx-cli/data has sample files",
        )
    })
}

/// The header and rows of CSV text.
fn parse_csv(reader: impl Read, source: &str) -> Result<(Vec<String>, Vec<Vec<String>>), CliError> {
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(reader);
    let header: Vec<String> = reader
        .headers()
        .map_err(|e| {
            CliError::input(format!("{source}: {e}"), "the first line must name the columns")
        })?
        .iter()
        .map(str::to_string)
        .collect();
    let rows = reader
        .records()
        .map(|r| r.map(|rec| rec.iter().map(str::to_string).collect()))
        .collect::<Result<Vec<Vec<String>>, _>>()
        .map_err(|e| {
            CliError::input(format!("{source}: {e}"), "every row needs one value per column")
        })?;
    Ok((header, rows))
}

/// A cell as a finite number.
fn number(source: &str, row: usize, column: &str, cell: &str) -> Result<f64, CliError> {
    cell.parse::<f64>().ok().filter(|v| v.is_finite()).ok_or_else(|| {
        CliError::input(
            format!("{source}: row {row}, column `{column}`: `{cell}` is not a number"),
            "fix or remove that row",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn file(text: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(text.as_bytes()).unwrap();
        f
    }

    #[test]
    fn returns_are_read_with_or_without_a_date_column() {
        let f = file("date,A,B\n2026-01-02,0.01,0.02\n2026-01-03,-0.01,0.00\n");
        let (assets, m) = read_returns(f.path()).unwrap();
        assert_eq!(assets, vec!["A", "B"]);
        assert_eq!(m.dim(), (2, 2));
        let f = file("A,B\n0.01,0.02\n");
        assert_eq!(read_returns(f.path()).unwrap().1.dim(), (1, 2));
    }

    #[test]
    fn a_bad_cell_is_named_by_row_and_column() {
        let f = file("A,B\n0.01,0.02\n0.03,oops\n");
        let err = read_returns(f.path()).unwrap_err();
        assert!(err.to_string().contains("row 3, column `B`: `oops`"), "{err}");
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn a_ragged_or_empty_file_is_an_input_error() {
        assert!(read_returns(file("A,B\n0.01\n").path()).is_err());
        assert!(read_returns(file("A,B\n").path()).is_err());
        assert!(read_returns(Path::new("/no/such/file.csv"))
            .unwrap_err()
            .hint()
            .unwrap()
            .contains("sample"));
    }

    #[test]
    fn a_series_is_a_named_or_the_last_column() {
        let f = file("day,cash\n1,100\n2,110\n");
        assert_eq!(read_series(f.path(), None).unwrap(), vec![100.0, 110.0]);
        assert_eq!(read_series(f.path(), Some("day")).unwrap(), vec![1.0, 2.0]);
        let err = read_series(f.path(), Some("nope")).unwrap_err();
        assert!(err.hint().unwrap().contains("day, cash"));
    }
}
