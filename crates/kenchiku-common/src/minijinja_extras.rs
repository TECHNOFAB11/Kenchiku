pub mod filters {
    use chrono::DateTime;

    pub fn timeformat(ts: i64, format: Option<&str>) -> String {
        let format = format.unwrap_or("%d/%m/%Y %H:%M");
        let datetime = DateTime::from_timestamp(ts, 0);
        if let Some(dt) = datetime {
            dt.format(format).to_string()
        } else {
            "".to_string()
        }
    }
}

pub mod functions {
    pub fn panic(message: String) -> Result<(), minijinja::Error> {
        Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            format!("Panicked: {}", message),
        ))
    }

    pub fn now() -> i64 {
        chrono::Utc::now().timestamp()
    }
}

pub fn register(mut env: minijinja::Environment) -> minijinja::Environment {
    env.add_filter("timeformat", filters::timeformat);
    env.add_function("panic", functions::panic);
    env.add_function("now", functions::now);

    env
}
