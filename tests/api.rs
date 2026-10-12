//! Every method against a mock Zalo API.
#![allow(clippy::unwrap_used)] // test code: a panic is the failure report

use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};
use zalo_bot::methods::SendMessage;
use zalo_bot::{
    Bot, ChatAction, ChatType, Content, Error, EventName, ParseMode, PollOptions, Style, TextStyle,
    WebhookOutcome,
};

const TOKEN: &str = "1234567:s3cr3t-token";

fn ok(result: Value) -> ResponseTemplate {
    #![allow(clippy::needless_pass_by_value)] // reads better at call sites
    ResponseTemplate::new(200).set_body_json(json!({ "ok": true, "result": result }))
}

fn api_error(code: i64, description: &str) -> ResponseTemplate {
    // Zalo reports errors with HTTP 200.
    ResponseTemplate::new(200)
        .set_body_json(json!({ "ok": false, "error_code": code, "description": description }))
}

fn text_update(text: &str) -> Value {
    json!({
        "event_name": "message.text.received",
        "message": {
            "from": { "id": "u1", "display_name": "Ted", "is_bot": false },
            "chat": { "id": "c1", "chat_type": "PRIVATE" },
            "text": text, "message_id": "m1", "date": 1_750_316_131_602_i64
        }
    })
}

async fn setup() -> (MockServer, Bot) {
    let server = MockServer::start().await;
    let bot = Bot::new(TOKEN).with_base_url(server.uri());
    (server, bot)
}

fn at(m: &str) -> wiremock::matchers::PathExactMatcher {
    path(format!("/bot{TOKEN}/{m}"))
}

#[tokio::test]
async fn get_me() {
    let (server, bot) = setup().await;
    Mock::given(method("POST"))
        .and(at("getMe"))
        .respond_with(ok(json!({
            "id": "1234567", "account_name": "bot.shop", "account_type": "BASIC", "can_join_groups": true
        })))
        .expect(1)
        .mount(&server)
        .await;
    let me = bot.get_me().await.unwrap();
    assert_eq!(me.account_name, "bot.shop");
    assert!(me.can_join_groups);
}

#[tokio::test]
async fn send_message_with_formatting() {
    let (server, bot) = setup().await;
    Mock::given(at("sendMessage"))
        .and(body_json(json!({
            "chat_id": "c1", "text": "Xin chào bạn", "text_styles": [{"start": 4, "len": 4, "st": ["b", "c_db342e"]}]
        })))
        .respond_with(ok(json!({ "message_id": "m9", "date": 1 })))
        .expect(1)
        .mount(&server)
        .await;
    let style = TextStyle::on("Xin chào bạn", "chào", [Style::Bold, Style::Red]).unwrap();
    let sent = bot
        .send(SendMessage::new("c1", "Xin chào bạn").style(style))
        .await
        .unwrap();
    assert_eq!(sent.message_id, "m9");

    Mock::given(at("sendMessage"))
        .and(body_json(
            json!({ "chat_id": "c1", "text": "**hi**", "parse_mode": "markdown" }),
        ))
        .respond_with(ok(json!({ "message_id": "m10", "date": 2 })))
        .mount(&server)
        .await;
    bot.send(SendMessage::new("c1", "**hi**").parse_mode(ParseMode::Markdown))
        .await
        .unwrap();
}

#[tokio::test]
async fn media_and_actions() {
    let (server, bot) = setup().await;
    for (m, body) in [
        (
            "sendPhoto",
            json!({"chat_id": "c1", "photo": "https://x/p.jpg", "caption": "cap"}),
        ),
        (
            "sendSticker",
            json!({"chat_id": "c1", "sticker": "0e078a2fb66a5f34067b"}),
        ),
        (
            "sendVoice",
            json!({"chat_id": "c1", "voice_url": "https://x/v.aac"}),
        ),
    ] {
        Mock::given(at(m))
            .and(body_json(body))
            .respond_with(ok(json!({ "message_id": m, "date": 1 })))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(at("sendChatAction"))
        .and(body_json(json!({"chat_id": "c1", "action": "typing"})))
        // The documented answer has no `result`.
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true })))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        bot.send_photo("c1", "https://x/p.jpg", Some("cap".into()))
            .await
            .unwrap()
            .message_id,
        "sendPhoto"
    );
    bot.send_sticker("c1", "0e078a2fb66a5f34067b")
        .await
        .unwrap();
    bot.send_voice("c1", "https://x/v.aac").await.unwrap();
    bot.send_chat_action("c1", ChatAction::Typing)
        .await
        .unwrap();
}

#[tokio::test]
async fn webhook_methods() {
    let (server, bot) = setup().await;
    Mock::given(at("setWebhook"))
        .and(body_json(
            json!({"url": "https://bot.example.com/zalo", "secret_token": "12345678"}),
        ))
        .respond_with(ok(json!({
            "url": "https://bot.example.com/zalo", "updated_at": 5,
            "verification": {"ok": false, "url": "https://bot.example.com/zalo",
                             "outcome": "webhook.err.tls", "hint": "fix the certificate"}
        })))
        .mount(&server)
        .await;
    Mock::given(at("getWebhookInfo"))
        .respond_with(ok(
            json!({"url": "https://bot.example.com/zalo", "updated_at": 5}),
        ))
        .mount(&server)
        .await;
    Mock::given(at("deleteWebhook"))
        .respond_with(ok(json!({"url": "", "updated_at": 6})))
        .mount(&server)
        .await;
    Mock::given(at("testWebhook"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"ok": false, "errorCode": 426, "description": "daily limit"}),
            ),
        )
        .mount(&server)
        .await;

    let set = bot
        .set_webhook("https://bot.example.com/zalo", "12345678")
        .await
        .unwrap();
    let check = set.verification.unwrap();
    assert!(!check.ok);
    assert_eq!(check.outcome, WebhookOutcome::Tls);
    assert!(bot.get_webhook_info().await.unwrap().is_set());
    assert!(!bot.delete_webhook().await.unwrap().is_set());
    let error = bot.test_webhook().await.unwrap_err();
    assert_eq!(error.code(), Some(426), "errorCode is read too");
    assert!(error.is_rate_limited());
}

#[tokio::test]
async fn get_updates_shapes() {
    let (server, bot) = setup().await;
    let answers = [
        ok(text_update("one")),
        ok(json!([text_update("two"), text_update("three")])),
        ok(Value::Null),
        ResponseTemplate::new(200).set_body_json(json!({"ok": true})),
        api_error(408, "Request timeout"),
    ];
    Mock::given(at("getUpdates"))
        .and(body_json(json!({"timeout": 1})))
        .respond_with(Sequence::new(answers))
        .mount(&server)
        .await;
    let t = Duration::from_secs(1);
    assert_eq!(bot.get_updates(t).await.unwrap().len(), 1);
    assert_eq!(bot.get_updates(t).await.unwrap().len(), 2);
    for _ in 0..3 {
        assert_eq!(bot.get_updates(t).await.unwrap(), []);
    }
}

#[tokio::test]
async fn updates_of_every_kind() {
    let raw = json!([
        {"event_name": "message.image.received", "message": {
            "from": {"id": "u", "display_name": "A", "is_bot": false},
            "chat": {"id": "g1", "chat_type": "GROUP"},
            "photo": "https://x/i.jpg", "caption": "look", "message_id": "1", "date": 1}},
        {"event_name": "message.sticker.received", "message": {
            "from": {"id": "u"}, "chat": {"id": "c"}, "sticker": "s1", "url": "https://x/s.png"}},
        {"event_name": "message.voice.received", "message": {
            "from": {"id": "u"}, "chat": {"id": "c"}, "voice_url": "https://x/v.aac"}},
        {"event_name": "message.unsupported.received", "message": {
            "from": {"id": "u"}, "chat": {"id": "c", "chat_type": "SOMETHING_NEW"}}},
        {"event_name": "message.reaction.received", "message": {
            "from": {"id": "u"}, "chat": {"id": "c"}, "extra": 1}}
    ]);
    let updates: Vec<zalo_bot::Update> = serde_json::from_value(raw).unwrap();
    assert_eq!(
        updates[0].content(),
        Content::Image {
            url: "https://x/i.jpg",
            caption: Some("look")
        }
    );
    assert!(updates[0].message.is_group());
    assert_eq!(
        updates[1].content(),
        Content::Sticker {
            id: "s1",
            url: Some("https://x/s.png")
        }
    );
    assert_eq!(
        updates[2].content(),
        Content::Voice {
            url: "https://x/v.aac"
        }
    );
    assert_eq!(updates[3].content(), Content::Unsupported);
    assert_eq!(
        updates[3].message.chat.chat_type,
        ChatType::Other("SOMETHING_NEW".into())
    );
    assert_eq!(
        updates[4].event_name,
        EventName::Other("message.reaction.received".into())
    );
    assert_eq!(updates[4].content(), Content::Unknown);
}

#[tokio::test]
async fn errors_are_typed_and_never_show_the_token() {
    let (server, bot) = setup().await;
    Mock::given(at("getMe"))
        .respond_with(api_error(401, "Unauthorized"))
        .mount(&server)
        .await;
    let error = bot.get_me().await.unwrap_err();
    assert!(error.is_unauthorized());
    assert!(!error.is_retryable());

    // A transport error: nothing listens on this port.
    let offline = Bot::new(TOKEN).with_base_url("http://127.0.0.1:9");
    let error = offline.get_me().await.unwrap_err();
    assert!(matches!(error, Error::Http(_)));
    assert!(error.is_retryable());
    let shown = format!("{error} {error:?} {offline:?}");
    assert!(!shown.contains("s3cr3t"), "{shown}");
    assert!(format!("{offline:?}").contains("1234567:***"));
}

#[tokio::test]
async fn invalid_requests_are_not_sent() {
    let (server, bot) = setup().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ok(json!({})))
        .expect(0)
        .mount(&server)
        .await;
    for error in [
        bot.send_message("c1", "").await.unwrap_err(),
        bot.send_message("c1", "x".repeat(2001)).await.unwrap_err(),
        bot.send_message("", "hi").await.unwrap_err(),
        bot.set_webhook("http://insecure.example.com", "12345678")
            .await
            .unwrap_err(),
        bot.set_webhook("https://ok.example.com", "short")
            .await
            .unwrap_err(),
        bot.send_photo("c1", "not a url", None).await.unwrap_err(),
    ] {
        assert!(matches!(error, Error::Invalid(_)), "{error}");
    }
    // 2000 characters (not bytes) are fine.
    Mock::given(at("sendMessage"))
        .respond_with(ok(json!({"message_id": "m", "date": 0})))
        .mount(&server)
        .await;
}

#[tokio::test]
async fn polling_stream() {
    let (server, bot) = setup().await;
    Mock::given(at("getUpdates"))
        .respond_with(Sequence::new([
            ok(text_update("a")),
            ok(Value::Null),
            api_error(409, "Conflict: webhook is set"),
            ok(json!([text_update("b"), text_update("c")])),
            api_error(401, "Unauthorized"),
        ]))
        .mount(&server)
        .await;
    Mock::given(at("deleteWebhook"))
        .respond_with(ok(json!({"url": "", "updated_at": 1})))
        .expect(1)
        .mount(&server)
        .await;
    let options = PollOptions::default()
        .timeout(Duration::from_secs(1))
        .delete_webhook(true);
    let items: Vec<_> = bot.updates(options).collect().await;
    let texts: Vec<_> = items
        .iter()
        .filter_map(|i| i.as_ref().ok())
        .map(|u| u.message.text.clone().unwrap())
        .collect();
    assert_eq!(texts, ["a", "b", "c"]);
    assert!(
        items
            .last()
            .unwrap()
            .as_ref()
            .unwrap_err()
            .is_unauthorized()
    );
}

/// Answers with each template in turn, then repeats the last.
struct Sequence {
    answers: Vec<ResponseTemplate>,
    next: std::sync::atomic::AtomicUsize,
}

impl Sequence {
    fn new(answers: impl IntoIterator<Item = ResponseTemplate>) -> Self {
        Self {
            answers: answers.into_iter().collect(),
            next: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Respond for Sequence {
    fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
        let i = self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.answers[i.min(self.answers.len() - 1)].clone()
    }
}
