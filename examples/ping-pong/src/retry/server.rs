use anysystem::{Context, Message, Process};

#[derive(Clone)]
pub struct RetryPingServer {}

impl Process for RetryPingServer {
    fn on_message(&mut self, msg: Message, from: String, ctx: &mut Context) -> Result<(), String> {
        if msg.tip == "PING" {
            let resp = Message::new("PONG".to_string(), msg.data);
            ctx.send(resp, from);
        }
        Ok(())
    }

    fn on_local_message(&mut self, _msg: Message, _ctx: &mut Context) -> Result<(), String> {
        Ok(())
    }

    fn on_timer(&mut self, _timer: String, _ctx: &mut Context) -> Result<(), String> {
        Ok(())
    }
}
