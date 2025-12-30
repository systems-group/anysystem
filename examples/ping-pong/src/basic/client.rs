use anysystem::{Context, Message, Process};

#[derive(Clone)]
pub struct BasicPingClient {
    _id: String,
    server: String,
}

impl BasicPingClient {
    pub fn new(id: &str, server: &str) -> Self {
        Self {
            _id: id.to_string(),
            server: server.to_string(),
        }
    }
}

impl Process for BasicPingClient {
    fn on_message(&mut self, msg: Message, _from: String, ctx: &mut Context) -> Result<(), String> {
        if msg.tip == "PONG" {
            ctx.send_local(msg);
        }
        Ok(())
    }

    fn on_local_message(&mut self, msg: Message, ctx: &mut Context) -> Result<(), String> {
        if msg.tip == "PING" {
            ctx.send(msg, self.server.clone());
        }
        Ok(())
    }

    fn on_timer(&mut self, _timer: String, _ctx: &mut Context) -> Result<(), String> {
        Ok(())
    }
}
