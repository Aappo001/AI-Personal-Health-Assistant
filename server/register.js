document.querySelector('#register-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const username = document.querySelector('#username').value;
    const email = document.querySelector('#email').value;
    const password = document.querySelector('#password').value;
    const firstName = document.querySelector('#first_name').value;
    const lastName = document.querySelector('#last_name').value;
    
    const response = await fetch('http://localhost:3000/register', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            username,
            email,
            password,
            first_name: firstName,
            last_name: lastName
        })
    });
    
    const result = await response.json();
    
    if (result.success) {
        alert('User registered successfully!');
    } else {
        alert('Registration failed: ' + result.message);
    }
});
