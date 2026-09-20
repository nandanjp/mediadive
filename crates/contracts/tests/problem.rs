//! The error envelope is a contract; `type` and omitted `errors` are load-bearing.

use mediadive_contracts::Problem;

#[test]
fn serializes_type_and_omits_absent_errors() {
    let problem = Problem {
        kind: "about:blank".into(),
        title: "Unprocessable Entity".into(),
        status: 422,
        detail: "Registration could not be completed.".into(),
        code: "email_already_registered".into(),
        errors: None,
    };

    let json = serde_json::to_value(&problem).expect("serialize");
    assert_eq!(json["type"], "about:blank", "`kind` serializes as `type`");
    assert_eq!(json["code"], "email_already_registered");
    assert!(
        json.get("errors").is_none(),
        "absent field errors are omitted, not null"
    );
}
