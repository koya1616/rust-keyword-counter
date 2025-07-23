interface User {
    name: string;
    age: number;
    email?: string;
}

type ID = string | number;

class UserService {
    private users: User[] = [];

    constructor() {
        this.users = [];
    }

    async findUser(id: ID): Promise<User | undefined> {
        return this.users.find(user => user.name === id);
    }

    addUser(user: User): void {
        this.users.push(user);
    }
}

const service = new UserService();
export default service;