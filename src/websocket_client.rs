use super::react_to_message;
use ws::{CloseCode, Frame, Handshake, OpCode, Sender};
use ws::util::{Timeout, Token};

const PING: Token = Token(1);
const PING_TIMEOUT: u64 = 10_000;
const EXPIRE: Token = Token(2);
const EXPIRE_TIMEOUT: u64 = 60_000;

lazy_static! {
    /// A special value used for the Ping messages.
    static ref PING_PONG: Vec<u8> = Vec::from("mattermost-client".as_bytes());
}

pub struct WsClient {
    pub ws: Sender,
    pub timeout: Option<Timeout>,
    pub own_id: Option<String>,
    pub token: String,
    pub servername: String,
    pub mobile_number: String,
}

use ws::{Error, ErrorKind, Result};
impl ::ws::Handler for WsClient {
    fn on_message(&mut self, msg: ::ws::Message) -> Result<()> {
        if msg.is_text() {
            let msg = msg.into_text().expect("Must be text");
            react_to_message(self, &msg);
        }
        Ok(())
    }

    fn on_open(&mut self, _: Handshake) -> Result<()> {
        // schedule a timeout to send a ping every 5 seconds
        self.ws.timeout(PING_TIMEOUT, PING)?;
        // schedule a timeout to close the connection if there is no activity for 30 seconds
        self.ws.timeout(EXPIRE_TIMEOUT, EXPIRE)
    }

    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            PING => {
                debug!("WS: Perform ping");
                self.ws.ping(PING_PONG.clone())?;
                self.ws.timeout(PING_TIMEOUT, PING)
            }
            EXPIRE => self.ws.close(CloseCode::Away),
            _ => Err(Error::new(
                ErrorKind::Internal,
                "Invalid timeout token encountered!",
            )),
        }
    }

    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> Result<()> {
        if event == EXPIRE {
            if let Some(t) = self.timeout.take() {
                debug!("WS: Cancel expire timeout");
                self.ws.cancel(t)?
            }
            self.timeout = Some(timeout)
        }
        Ok(())
    }

    fn on_frame(&mut self, frame: Frame) -> Result<Option<Frame>> {
        debug!("Handler received: {}", frame);
        // default implementation doesn't allow for reserved bits to be set
        if frame.has_rsv1() || frame.has_rsv2() || frame.has_rsv3() {
            Err(Error::new(
                ErrorKind::Protocol,
                "Encountered frame with reserved bits set.",
            ))
        } else {
            if frame.opcode() == OpCode::Pong && frame.payload() == &*PING_PONG {
                debug!("WS: Received pong");
                // reset timeout if ping/pong was successful
                self.ws.timeout(EXPIRE_TIMEOUT, EXPIRE)?
            }
            Ok(Some(frame))
        }
    }
}
