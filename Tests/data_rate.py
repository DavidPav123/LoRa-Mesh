import pyautogui
import time

if __name__ == "__main__":
    MESSAGES_PER_MINUTE: int = 5
    WAIT_TIME: float = 60 / MESSAGES_PER_MINUTE

    # Time to click on application
    time.sleep(5)

    # Send the set number of messages with a wait inbettwen that adds up to 60 seconds
    for _ in range(MESSAGES_PER_MINUTE):
        pyautogui.press("enter")
        time.sleep(WAIT_TIME)

    print("Finished")
