const submitForm = async () => {
    const formData = {
        username: document.getElementById("username").value,
        password: document.getElementById("password").value,
        email: document.getElementById("email").value, // New field for email
        first_name: document.getElementById("first_name").value, // New field for first name
        last_name: document.getElementById("last_name").value // New field for last name
    };

    try {
        const response = await fetch("http://localhost:3000/users/create", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(formData),
        });

        if (!response.ok) {
            const errorMessage = await response.text();
            throw new Error(`Error: ${errorMessage}`);
        }

        const result = await response.json();
        console.log("User created successfully:", result);
        
    } catch (error) {
        console.error("Failed to create user:", error);
      
    }
};

document.getElementById("registrationForm").addEventListener("submit", (e) => {
    e.preventDefault(); 
    submitForm();
});
