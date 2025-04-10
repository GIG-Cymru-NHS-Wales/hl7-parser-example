fn main() -> std::io::Result<()> {
    let input: String = std::io::read_to_string(std::io::stdin())?;
    println!("{input}");
    let message = hl7_parser::Message::parse(input.as_str()).expect("Message::parse");
    println!("{:#?}", message);
    Ok(())
}

#[cfg(test)]
mod tests {

    // Examples thanks to the crate documentation:
    // https://crates.io/crates/hl7-parser
    //
    // These examples are here to help you understand how to use the crate.
    // We hope to add more examples as we go that are for more-complex goals.

    #[test]
    fn query_message() {
        let input = "MSH|^~\\&|foo|bar|baz|quux|20010504094523||ADT^A01|1234|P|2.3|||";
        let message = hl7_parser::Message::parse(input).unwrap();
        let field = message.query("MSH.3").unwrap().raw_value();
        assert_eq!(field, "foo");
        let component = message.query("MSH.7.1").unwrap().raw_value();
        assert_eq!(component, "20010504094523");
    }

    #[test]
    fn locate_cursor_within_message() {
        let input = "MSH|^~\\&|asdf\rPID|1|0";
        let message = hl7_parser::Message::parse(input).unwrap();
        let cursor = hl7_parser::locate::locate_cursor(&message, 19).expect("cursor is located");
        assert_eq!(cursor.segment.unwrap().0, "PID");
        assert_eq!(cursor.segment.unwrap().1, 1);
        assert_eq!(cursor.field.unwrap().0, 1);
        assert_eq!(cursor.field.unwrap().1.raw_value(), "1");
    }

    #[test]
    fn decode_encoded_string() {
        let separators = hl7_parser::message::Separators::default(); // or, from a parsed message
        let input = "foo|bar^baz&quux~quuz\\corge\rquack\nduck";
        let expect = r"foo\F\bar\S\baz\T\quux\R\quuz\E\corge\X0D\quack\X0A\duck";
        let actual = separators.encode(input).to_string();
        assert_eq!(actual, expect);
    }

    #[test]
    fn parse_timestamp() {
        let ts: hl7_parser::datetime::TimeStamp =
            hl7_parser::datetime::parse_timestamp("20230312195905.1234-0700", false)
                .expect("can parse timestamp");
        assert_eq!(ts.year, 2023);
        assert_eq!(ts.month, Some(3));
        assert_eq!(ts.day, Some(12));
        assert_eq!(ts.hour, Some(19));
        assert_eq!(ts.minute, Some(59));
        assert_eq!(ts.second, Some(5));
        assert_eq!(ts.microsecond, Some(123_400));
        assert_eq!(
            ts.offset,
            Some(hl7_parser::datetime::TimeStampOffset {
                hours: -7,
                minutes: 0,
            })
        );
    }

    #[test]
    fn build_message() {
        use hl7_parser::builder::prelude::*;
        let message = MessageBuilder::new(Separators::default())
            .with_segment(
                SegmentBuilder::new("MSH")
                    .with_field_value(3, "SendingApp")
                    .with_field_value(4, "SendingFac")
                    .with_field_value(5, "ReceivingApp")
                    .with_field_value(6, "ReceivingFac")
                    .with_field(
                        9,
                        FieldBuilder::default()
                            .with_component(1, "ADT")
                            .with_component(2, "A01"),
                    )
                    .with_field_value(10, "123456")
                    .with_field_value(11, "P")
                    .with_field_value(12, "2.3"),
            )
            .with_segment(
                SegmentBuilder::new("PID")
                    .with_field_value(3, "123456")
                    .with_field(
                        5,
                        FieldBuilder::default()
                            .with_component(1, "Doe")
                            .with_component(2, "John"),
                    )
                    .with_field_value(7, "19700101"),
            )
            .render_with_newlines()
            .to_string();
        let expect = "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|||ADT^A01|123456|P|2.3\nPID|||123456||Doe^John||19700101";
        assert_eq!(message, expect);
    }

}
