# Proyecto de Servidor en Rust

## Tareas Pendientes

### Configuración y Mejora del Servidor

- [x] Todos los path para usan el handler para etag
- [x] Configurar caché y su duración (cabecera de respuesta HTTP) y código 304, y el ETag para archivos estáticos: css, js, etc.
- [ ] Configurar manejador global de errores y que devuelva error 500 con su página.
- [x] Configurar todos los errores 400.
- [x] Dejar preparadas las redirecciones 301 y 302.
- [ ] Configurar cómo se establece la longitud máxima de URL (buscar su error) y hacer pruebas con diferentes longitudes.
- [ ] Configurar tamaño máximo de petición y tamaño máximo de cabecera para peticiones GET, POST, etc. (error 451).
- [ ] Configurar las conexiones máximas simultáneas globales, los threads que se van a usar y los workers. Consultar si Rust proporciona un monitor para revisar conexiones o sockets usados/libres en tiempo real.
- [ ] Configurar respuesta HEADER para cada URL que se active.
- [ ] Configurar limitaciones CORS y HSTS y protección contra XSS, clickjacking y sniffing (preguntar si falta alguna protección por añadir).
- [ ] Establecer el número máximo de conexiones abiertas por cliente y protección contra ataques DOS (429 Too Many Requests).
- [ ] Configurar el timeout máximo de cada conexión.
