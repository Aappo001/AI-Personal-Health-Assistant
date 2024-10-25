// Define the BiometricData class
public class BiometricData {
    private int heartRate;
    private int sleepHours;

    // Constructor
    public BiometricData(int heartRate, int sleepHours) {
        this.heartRate = heartRate;
        this.sleepHours = sleepHours;
    }

    // Getters and setters
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

    @Override
    public String toString() {
        return "BiometricData [heartRate=" + heartRate + ", sleepHours=" + sleepHours + "]";
    }
}
