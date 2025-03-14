//! Simple tests for the HTML templating system.

use std::borrow::Cow;

const SAMPLE_HTML: &[u8] = br##"<html>
<head>
    <title>Hello world</title>
</head>
<body>
    <script type="application/spa-cfg">{ "key": "value" }</script>
</body>
</html>"##;

const SAMPLE_TEMPLATE: &str = r##"{ "key": "value" }"##;
const TEMPLATE_SUBSTITUTION: &str = r##"{ "key": "other value" }"##;

const EXPECTED_HTML: &[u8] = br##"<html>
<head>
    <title>Hello world</title>
</head>
<body>
    <script type="application/spa-cfg">{ "key": "other value" }</script>
</body>
</html>"##;

fn assert_output_eq(left: &[u8], right: &[u8]) {
    let parse = |val: &[u8]| -> String {
        let val = std::str::from_utf8(val).unwrap();
        val.replace("\n", "")
    };

    let left = parse(left);
    let right = parse(right);

    assert_eq!(left, right);
}

#[test]
fn happy_path() {
    let happy_path_content_processor = |input: &str| -> Result<String, std::convert::Infallible> {
        assert_eq!(input, SAMPLE_TEMPLATE);
        Ok(TEMPLATE_SUBSTITUTION.into())
    };

    let processor = html_templating::Processor {
        template_element_filter: html_templating::template_element_filter::ScriptTag {
            script_type: Cow::Borrowed("application/spa-cfg"),
        },
        content_processor: happy_path_content_processor,
    };

    let output = processor.process(SAMPLE_HTML).unwrap();

    assert_output_eq(EXPECTED_HTML, &output);
}
