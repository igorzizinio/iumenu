#[derive(Clone)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub comment: String,
    pub exec: String,
    pub keywords: String, // TODO: Implement this as list of string
    pub app_type: String,
    pub categories: String,
    pub no_display: bool,
    pub only_show_in: String,
    pub icon: String,
}

impl DesktopApp {
    pub fn new(
        id: String,
        name: String,
        comment: String,
        exec: String,
        keywords: String,
        app_type: String,
        categories: String,
        icon: String,
        no_display: bool,
        only_show_in: String,
    ) -> Self {
        Self {
            id,
            name,
            comment,
            exec,
            keywords,
            app_type,
            categories,
            icon,
            no_display,
            only_show_in,
        }
    }
}
