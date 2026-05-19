use arboard::Clipboard;

pub struct X1BriefClipboard {
    clipboard: Clipboard,
}

impl X1BriefClipboard {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            clipboard: Clipboard::new()?,
        })
    }

    pub fn get_text(&mut self) -> anyhow::Result<String> {
        Ok(self.clipboard.get_text()?)
    }
}
