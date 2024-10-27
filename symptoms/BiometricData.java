// Define the BiometricData class
public class BiometricData {
    private int heartRate;
    private int sleepHours;
    private int age;
    private double pounds;
    private String dateOfBirth;
    private String location;
    private String mood;
    private String gender;
    private boolean isUnhealthy;
    // Constructor
    public BiometricData(int heartRate, int sleepHours, int age, double pounds, String dateOfBirth, String location, String mood, String gender, boolean isUnhealthy) {
        this.heartRate = heartRate;
        this.sleepHours = sleepHours;
        this.age = age;
        this.pounds = pounds;
        this.dateOfBirth = dateOfBirth;
        this.location = location;
        this.mood = mood;
        this.gender = gender;
        this.isUnhealthy = isUnhealthy;
    }

    // Getters and setters
    public void setIsUnhealthy(){
        if(heartRate > 100 && sleepHours < 8){
            isUnhealthy = true;
        }
        else{
            isUnhealthy = false;
        }
    }
    public boolean getIsUnhealthy(){
        return isUnhealthy;
    }

    public int getHeartRate() {
        return heartRate;
    }

    public void setHeartRate(int heartRate) {
        this.heartRate = heartRate;
    }

    public int getSleepHours() {
        return sleepHours;
    }

    public void setSleepHours(int sleepHours) {
        this.sleepHours = sleepHours;
    }

    public String getMood() {
        return mood;
    }

    public void setMood(String mood) {
        this.mood = mood;
    }
    public int getAge() {
        return age;
    }

    public void setAge(int age) {
        this.age = age;
    }
    public String getLocation() {
        return location;
    }

    public void setLocation(String location) {
        this.location = location;
    }
    public String getDateOfBirth() {
        return dateOfBirth;
    }

    public void setDateOfBirth(String dateOfBirth) {
        this.dateOfBirth = dateOfBirth;
    }
    public String getGender() {
        return gender;
    }

    public void setGender(String gender) {
        this.gender = gender;
    }
    public double getPounds() {
        return pounds;
    }

    public void setPounds(double pounds) {
        this.pounds = pounds;
    }
    @Override
    public String toString() {
        return "BiometricData [heartRate=" + heartRate + ", sleepHours=" + sleepHours + "]";
    }
}
