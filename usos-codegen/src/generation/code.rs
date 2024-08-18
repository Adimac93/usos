pub(crate) enum CodegenItem {
    Line(String),
    Indent,
    StopIndent,
}

impl CodegenItem {
    pub(crate) fn line(s: impl Into<String>) -> Self {
        Self::Line(s.into())
    }
}

pub(crate) struct Code {
    items: Vec<CodegenItem>,
}

impl Code {
    pub fn new() -> Self {
        Code { items: Vec::new() }
    }

    pub fn line(mut self, s: impl Into<String>) -> Self {
        self.items.push(CodegenItem::line(s));
        self
    }

    pub fn indent(mut self) -> Self {
        self.items.push(CodegenItem::Indent);
        self
    }

    pub fn stop_indent(mut self) -> Self {
        self.items.push(CodegenItem::StopIndent);
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.extend(other.items);
        self
    }

    /// Panics if indent count goes negative.
    pub fn to_code_string(self) -> String {
        let mut tab_count = 0;
        self.items
            .into_iter()
            .fold(String::new(), |mut acc, item| match item {
                CodegenItem::Indent => {
                    tab_count += 1;
                    acc
                }
                CodegenItem::StopIndent => {
                    if tab_count == 0 {
                        panic!("Indent count is negative");
                    }
                    tab_count -= 1;
                    acc
                }
                CodegenItem::Line(line) => {
                    let tabs = "\t".repeat(tab_count);
                    acc.push_str(&format!("{tabs}{line}\n"));
                    acc
                }
            })
    }
}

impl Extend<CodegenItem> for Code {
    fn extend<T: IntoIterator<Item = CodegenItem>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_struct_to_code_gives_coorect_result() {
        let mut code = Code::new();
        code.extend(vec![
            CodegenItem::line("fn foo() -> i32 {"),
            CodegenItem::Indent,
            CodegenItem::line("5"),
            CodegenItem::StopIndent,
            CodegenItem::line("}"),
        ]);

        assert_eq!(code.to_code_string(), "fn foo() -> i32 {\n\t5\n}\n");
    }

    #[test]
    fn code_struct_high_level_api_gives_correct_result() {
        let code = Code::new()
            .line("fn foo() -> i32 {")
            .indent()
            .line("5")
            .stop_indent()
            .line("}");

        assert_eq!(code.to_code_string(), "fn foo() -> i32 {\n\t5\n}\n");
    }

    #[test]
    fn code_struct_nested_gives_correct_result() {
        let struct_code = Code::new()
            .line("struct Foo {")
            .indent()
            .line("first: String,")
            .line("second: i32,")
            .stop_indent()
            .line("}");

        let code = Code::new()
            .line("fn foo() {")
            .indent()
            .merge(struct_code)
            .stop_indent()
            .line("}");

        assert_eq!(
            code.to_code_string(),
            "fn foo() {\n\tstruct Foo {\n\t\tfirst: String,\n\t\tsecond: i32,\n\t}\n}\n"
        )
    }
}
