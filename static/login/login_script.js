// document.getElementById('login-form').addEventListener('submit', function (event) {
//     event.preventDefault(); // Evita el envío del formulario
//
//     const username = document.getElementById('username').value;
//     const password = document.getElementById('password').value;
//     const errorMessage = document.getElementById('error-message');
//
//     // Lógica de autenticación básica
//     if (username === "admin" && password === "1234") {
//         alert("Inicio de sesión exitoso");
//         errorMessage.style.display = "none";
//         // Aquí podrías redirigir a otra página
//     } else {
//         errorMessage.textContent = "Usuario o contraseña incorrectos";
//         errorMessage.style.display = "block";
//     }
// });

document.getElementById('login-form').addEventListener('submit', async function (event) {
    event.preventDefault(); // Evita el envío del formulario

    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    const errorMessage = document.getElementById('error-message');

    try {
        // Enviar credenciales al servidor
        const response = await fetch('/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, password }),
        });

        if (response.ok) {
            // Obtener el token del servidor
            const data = await response.json();
            const token = data.token;

            // Guardar el token en localStorage (o sessionStorage)
            localStorage.setItem('authToken', token);

            // Redirigir a index
            window.location.href = '/';
        } else {
            // Mostrar mensaje de error
            const errorText = await response.text();
            errorMessage.textContent = errorText || "Usuario o contraseña incorrectos";
            errorMessage.style.display = "block";
        }
    } catch (error) {
        console.error('Error en la autenticación:', error);
        errorMessage.textContent = "Ocurrió un error. Inténtalo de nuevo.";
        errorMessage.style.display = "block";
    }
});
