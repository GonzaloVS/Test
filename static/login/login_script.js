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

            // Guardar el token en localStorage
            localStorage.setItem('authToken', token);

            // Redirigir a la página principal
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

// Verificar autenticación en la carga de la página
window.onload = async function () {
    const authToken = localStorage.getItem('authToken');
    if (!authToken) {
        // Redirigir al login si no hay token
        window.location.href = '/login';
        return;
    }

    try {
        const response = await fetch('/items', {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${authToken}`, // Puedes incluir un token adicional si lo deseas
            },
        });

        if (response.status === 401) {
            // Redirigir al login si la sesión no es válida
            localStorage.removeItem('authToken');
            window.location.href = '/login';
        }
    } catch (error) {
        console.error('Error al verificar la sesión:', error);
        window.location.href = '/login';
    }
};
