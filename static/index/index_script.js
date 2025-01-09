async function fetchItems() {
  const token = localStorage.getItem('authToken');
  if (!token) {
    alert('No autorizado. Redirigiendo a login.');
    window.location.href = '/login';
    return;
  }

  try {
    // Solicitud al servidor para obtener los datos
    const response = await fetch('/items', {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
      },
    });

    if (response.ok) {
      // Procesar y mostrar los datos
      const items = await response.json();
      const table = document.getElementById('items-table').getElementsByTagName('tbody')[0];
      table.innerHTML = ''; // Limpiar contenido previo

      items.forEach(item => {
        const row = table.insertRow();
        row.insertCell(0).textContent = item.id;
        row.insertCell(1).textContent = item.name;
        row.insertCell(2).textContent = item.description;
      });
    } else if (response.status === 401) {
      alert('Sesión no válida. Redirigiendo a login.');
      window.location.href = '/login';
    } else {
      alert('Error al cargar los datos. Intenta nuevamente.');
    }
  } catch (error) {
    console.error('Error al obtener los datos:', error);
    alert('Ocurrió un error. Intenta nuevamente.');
  }
}

// Llamar a fetchItems cuando la página se cargue
window.onload = fetchItems;
