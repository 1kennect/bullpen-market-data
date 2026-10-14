import Footer from "@/components/site/Footer";
import Hero from "@/components/site/Hero";
import HowItWorks from "@/components/site/HowItWorks";
import Nav from "@/components/site/Nav";
import OpenSource from "@/components/site/OpenSource";
import Sources from "@/components/site/Sources";
import Tape from "@/components/site/Tape";

/** The landing page. */
const Home = () => (
  <div className="min-h-screen">
    <Nav />
    <main className="relative">
      <Hero />
      <Tape />
      <Sources />
      <HowItWorks />
      <OpenSource />
    </main>
    <Footer />
  </div>
);

export default Home;
