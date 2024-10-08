document.querySelector('#registerForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const username = document.querySelector('#username').value;
    const email = document.querySelector('#email').value;
    const password = document.querySelector('#password').value;
    const firstName = document.querySelector('#firstName').value;
    const lastName = document.querySelector('#lastName').value;
    
    const response = await fetch('http://localhost:3000/api/register', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            username,
            email,
            password,
            firstName,
            lastName
        })
    });
    
    const result = await response.json();
    
    if (response.ok) {
        alert('User registered successfully!');
    } else {
        alert('Registration failed: ' + result.message);
    }
});
