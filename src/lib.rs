use actix_web::HttpResponse;


pub trait AppOrScope {
    fn route(self, path: &str, route: actix_web::Route) -> Self;
    fn service<F: actix_web::dev::HttpServiceFactory + 'static>(self, factory: F) -> Self;
}

impl<T: actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>> AppOrScope for actix_web::App<T> {
    fn route(self, path: &str, route: actix_web::Route) -> Self {
        self.route(path, route)
    }

    fn service<F: actix_web::dev::HttpServiceFactory + 'static>(self, factory: F) -> Self {
        self.service(factory)
    }
}

impl<T: actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>> AppOrScope for actix_web::Scope<T> {
    fn route(self, path: &str, route: actix_web::Route) -> Self {
        self.route(path, route)
    }

    fn service<F: actix_web::dev::HttpServiceFactory + 'static>(self, factory: F) -> Self {
        self.service(factory)
    }
}



macro_rules! application {
    {
        $visibility:vis struct $type_name:ident;
        let middleware = [$($middlewares:expr),* $(,)?];
        let services = [$(  $(#[$($attrs:tt)+])*  $service_kinds:ident($($services:tt)+)),* $(,)?];
    } => {
        $visibility struct $type_name;
        impl $type_name {
            $visibility fn register<S, B, F1, Srv, F2>(app: ::actix_web::App<S>) -> ::actix_web::App<impl ::actix_web::dev::ServiceFactory<::actix_web::dev::ServiceRequest, Config = (), Error = ::actix_web::error::Error, InitError = (), Response = ::actix_web::dev::ServiceResponse<impl ::actix_web::body::MessageBody>>>
            where
                S: ::actix_web::dev::ServiceFactory<::actix_web::dev::ServiceRequest, Config = (), Error = ::actix_web::error::Error, InitError = (), Response = ::actix_web::dev::ServiceResponse<B>, Future = F1, Service = Srv>,
                B: ::actix_web::body::MessageBody,
                F1: 'static,
                Srv: ::actix_web::dev::Service<::actix_web::dev::ServiceRequest, Response = ::actix_web::dev::ServiceResponse<B>, Error = ::actix_web::error::Error, Future = F2>,
                F2: 'static
             {
                $(let app = app.wrap($middlewares);)*
                register_services!(app, $(  $(#[$($attrs)+])*  $service_kinds($($services)+)),*);
                app
            }
        }
    };
}
macro_rules! module {
    {
        $path:expr => $visibility:vis struct $type_name:ident;
        let middleware = [$($middlewares:expr),* $(,)?];
        let inner_services = [$(  $(#[$($inner_attrs:tt)+])*  $inner_service_kinds:ident($($inner_services:tt)+)),* $(,)?];
        let outer_services = [$(  $(#[$($outer_attrs:tt)+])*  $outer_service_kinds:ident($($outer_services:tt)+)),* $(,)?];
    } => {
        $visibility struct $type_name;
        impl $type_name {
            $visibility fn register<A: $crate::AppOrScope>(app: A) -> A {
                register_services!(app, $(  $(#[$($outer_attrs)+])*  $outer_service_kinds($($outer_services)+),)*);
                let app = app.service({
                    let scope = ::actix_web::web::scope($path)$(.wrap($middlewares))*;
                    register_services!(scope, $(  $(#[$($inner_attrs)+])*  $inner_service_kinds($($inner_services)+),)*);
                    scope
                });
                app
            }
        }
    };
}

macro_rules! register_services {
    ($app:ident $(,)?) => {
        
    };
    ($app:ident, $(#[$($this_attrs:tt)+])* service($service:expr), $($(#[$($attrs:tt)+])*  $service_kinds:ident($($services:tt)+)),* $(,)?) => {
        $(#[$($this_attrs)+])*
        let $app = $app.service($service);
        register_services!($app, $($(#[$($attrs)+])*  $service_kinds($($services)+),)*)
    };
    ($app:ident, $(#[$($this_attrs:tt)+])* route($((middleware [$($middlewares:expr),* $(,)?]))? $method:ident $path:expr => $handler:expr), $($(#[$($attrs:tt)+])*  $service_kinds:ident($($services:tt)+)),* $(,)?) => {
        $(#[$($this_attrs)+])*
        route!($app, $method, $path, $handler, [$($($middlewares),*)?]);
        register_services!($app, $($(#[$($attrs)+])*  $service_kinds($($services)+),)*)
    };
    ($app:ident, $(#[$($this_attrs:tt)+])* module($module:ty), $($(#[$($attrs:tt)+])*  $service_kinds:ident($($services:tt)+)),* $(,)?) => {
        $(#[$($this_attrs)+])*
        let $app = <$module>::register($app);
        register_services!($app, $($(#[$($attrs)+])*  $service_kinds($($services)+),)*)
    };
}

macro_rules! route {

    ($app:ident, GET, $path:expr, $handler:expr, [$($middlewares:expr),* $(,)?]) => {
        let $app = $app.route($path, ::actix_web::web::get().to($handler)$(.wrap($middlewares))*);
    };
    ($app:ident, POST, $path:expr, $handler:expr, [$($middlewares:expr),* $(,)?]) => {
        let $app = $app.route($path, ::actix_web::web::post().to($handler)$(.wrap($middlewares))*);
    };
    ($app:ident, ALL, $path:expr, $handler:expr, [$($middlewares:expr),* $(,)?]) => {
        let $app = $app.route($path, ::actix_web::web::to($handler)$(.wrap($middlewares))*);
    };
}

application! {
    pub struct MyApp;
    let middleware = [
        actix_web::middleware::NormalizePath::default(),
        actix_web::middleware::Logger::default(),
    ];
    let services = [
        #[cfg(feature = "static-files")] service(actix_files::Files::new("/", ".")),
        route(POST "/test/abc" => abc_handler),
        route((middleware [actix_web::middleware::ErrorHandlers::new()]) GET "/foo" => foobar),
        route(ALL "/whatever" => whatever_handler),
        module(Module),
    ];
}

module! {
    "/collection" => pub struct Module;
    let middleware = [
        actix_web::middleware::NormalizePath::default(),
        actix_web::middleware::Logger::default(),
        actix_web::middleware::ErrorHandlers::new()
    ];
    let inner_services = [
        route(ALL "/inner" => abc_handler)

    ];
    let outer_services = [
        route(ALL "/outer" => abc_handler)
    ];
}

async fn abc_handler() -> HttpResponse {
    todo!()
}

async fn whatever_handler() -> HttpResponse {
    todo!()
}

async fn foobar() -> HttpResponse {
    todo!()
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
